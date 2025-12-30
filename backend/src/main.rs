use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::Extension,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch, Notify};
use tracing::{info, warn};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use rustls::crypto::ring;
use axum::http::{header, StatusCode};
use axum::extract::Path as AxPath;
use axum::body::Body as HyperBody;
use tokio::fs;
use mime_guess;

mod telemetry;
use crate::telemetry::Pose;
mod broadcast;
use crate::broadcast as broadcast_mod;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    // Ensure rustls has a concrete crypto provider installed for this process.
    ring::default_provider()
        .install_default()
        .expect("failed to install rustls ring crypto provider");

    // latest-value telemetry path (watch channel)
    let (pose_tx, _pose_rx) = watch::channel(Pose { yaw: 0.0, pitch: 0.0, roll: 0.0 });

    // AI work queue (bounded)
    let (ai_tx, ai_rx) = mpsc::channel::<String>(16);

    // Persistence queue (bounded)
    let (persist_tx, persist_rx) = mpsc::channel::<String>(32);

    let slow_inference = Arc::new(AtomicBool::new(false));

    // Simple observable metrics
    let telemetry_counter = Arc::new(AtomicU64::new(0));
    let ai_queue_len = Arc::new(AtomicU64::new(0));
    let persist_queue_len = Arc::new(AtomicU64::new(0));

    // Broadcast channels for PC->client streams (pose, voice)
    let (pose_bcast_tx, _pose_bcast_rx) = tokio::sync::broadcast::channel::<String>(1024);
    let (voice_bcast_tx, _voice_bcast_rx) = tokio::sync::broadcast::channel::<String>(1024);

    // Spawn actors
    tokio::spawn(telemetry_processor_task(pose_tx.subscribe(), telemetry_counter.clone()));
    tokio::spawn(audio_buffer_task());
    tokio::spawn(ai_orchestrator_task(ai_rx, persist_tx.clone(), slow_inference.clone(), ai_queue_len.clone()));
    tokio::spawn(persistence_worker_task(persist_rx, persist_queue_len.clone()));

    #[cfg(feature = "persistence")]
    let store = {
        use std::path::PathBuf;
        let path = PathBuf::from("data/rocksdb");
        std::fs::create_dir_all(&path)?;
        let s = storage::RocksStore::open(path)?;
        Some(std::sync::Arc::new(s))
    };

    // Optional synthetic telemetry generator (env var: SYNTHETIC_TELEMETRY=1)
    if std::env::var("SYNTHETIC_TELEMETRY").as_deref() == Ok("1") {
        info!("starting synthetic 60Hz telemetry generator");
        let gen_tx = pose_tx.clone();
        tokio::spawn(async move {
            let mut t = tokio::time::interval(std::time::Duration::from_micros(16_667)); // ~60Hz
            loop {
                t.tick().await;
                let now = Instant::now();
                let pose = Pose { yaw: now.elapsed().as_secs_f32() % 1.0, pitch: 0.0, roll: 0.0 };
                let _ = gen_tx.send(pose);
            }
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get({
            let telemetry_counter = telemetry_counter.clone();
            let ai_queue_len = ai_queue_len.clone();
            let persist_queue_len = persist_queue_len.clone();
            move || async move {
                let lines = format!(
                    "telemetry_received {}\nai_queue_len {}\npersist_queue_len {}\n",
                    telemetry_counter.load(Ordering::Relaxed),
                    ai_queue_len.load(Ordering::Relaxed),
                    persist_queue_len.load(Ordering::Relaxed)
                );
                (axum::http::StatusCode::OK, lines)
            }
        }))
        .route("/ws/telemetry", get(telemetry::ws_telemetry_handler))
        // Ingest endpoints (PC side) — connectors for native sidecars
        .route("/ws/pose-ingest", get(broadcast_mod::ws_pose_ingest))
        .route("/ws/voice-ingest", get(broadcast_mod::ws_voice_ingest))
        // Client subscription endpoints (iPhone viewport)
        .route("/ws/pose", get(broadcast_mod::ws_pose_client))
        .route("/ws/voice", get(broadcast_mod::ws_voice_client))
        .route("/toggle_slow_inference", get({
            let slow_inference = slow_inference.clone();
            move || async move {
                let prev = slow_inference.fetch_xor(true, Ordering::SeqCst);
                let state = if prev { "off" } else { "on" };
                (axum::http::StatusCode::OK, format!("slow_inference {}", state))
            }
        }))
        .route("/enqueue_ai", axum::routing::post({
            let ai_tx = ai_tx.clone();
            let ai_queue_len = ai_queue_len.clone();
            move |Json(payload): Json<serde_json::Value>| {
                let ai_tx = ai_tx.clone();
                let ai_queue_len = ai_queue_len.clone();
                async move {
                    let s = payload.to_string();
                    match ai_tx.try_send(s) {
                        Ok(()) => {
                            ai_queue_len.fetch_add(1, Ordering::Relaxed);
                            (axum::http::StatusCode::ACCEPTED, "enqueued")
                        }
                        Err(_) => (axum::http::StatusCode::SERVICE_UNAVAILABLE, "queue full"),
                    }
                }
            }
        }))
        .route("/", get(|| async { "AURA-1 backend prototype" }));

    #[cfg(feature = "persistence")]
    let app = {
        let store = store.expect("persistence store created");
        app
            .route("/append_log", axum::routing::post({
                let store = store.clone();
                move |Json(payload): Json<serde_json::Value>| {
                    let store = store.clone();
                    async move {
                        let speaker = payload.get("speaker").and_then(|v| v.as_str()).unwrap_or("user");
                        let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        match store.append_log_atomic(speaker, content) {
                            Ok(id) => (axum::http::StatusCode::CREATED, serde_json::json!({"id": id}).to_string()),
                            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                        }
                    }
                }
            }))
            .route("/log/:id", get({
                let store = store.clone();
                move |axum::extract::Path(id): axum::extract::Path<u64>| {
                    let store = store.clone();
                    async move {
                        match store.get_log(id) {
                            Ok(Some(le)) => (axum::http::StatusCode::OK, serde_json::to_string(&le).unwrap()),
                            Ok(None) => (axum::http::StatusCode::NOT_FOUND, "not found".to_string()),
                            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                        }
                    }
                }
            }))
            .route("/mmr_root", get({
                let store = store.clone();
                move || {
                    let store = store.clone();
                    async move {
                        match store.get_root() {
                            Ok(Some(r)) => (axum::http::StatusCode::OK, base64::encode(r)),
                            Ok(None) => (axum::http::StatusCode::OK, "".to_string()),
                            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                        }
                    }
                }
            }))
            .route("/prove/:id", get({
                let store = store.clone();
                move |axum::extract::Path(id): axum::extract::Path<u64>| {
                    let store = store.clone();
                    async move {
                        match store.prove(id) {
                            Ok(Some((leaf, peaks))) => {
                                let resp = serde_json::json!({"leaf": base64::encode(leaf), "peaks": peaks});
                                (axum::http::StatusCode::OK, serde_json::to_string(&resp).unwrap())
                            }
                            Ok(None) => (axum::http::StatusCode::NOT_FOUND, "not found".to_string()),
                            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                        }
                    }
                }
            }))
            .route("/snapshot", axum::routing::post({
                let store = store.clone();
                move |Json(payload): Json<serde_json::Value>| {
                    let store = store.clone();
                    async move {
                        let path = payload.get("path").and_then(|v| v.as_str()).unwrap_or("data/checkpoint");
                        match store.create_snapshot(Path::new(path)) {
                            Ok(()) => (axum::http::StatusCode::OK, format!("snapshot created: {}", path)),
                            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                        }
                    }
                }
            }))
        
    };

    // Attach shared state as axum `Extension`s so handlers can extract them.
    let app = app
        .layer(Extension(pose_tx.clone()))
        .layer(Extension(telemetry_counter.clone()))
        .layer(Extension(pose_bcast_tx.clone()))
        .layer(Extension(voice_bcast_tx.clone()));

    // Serve static TTS/audio files from `backend/data/audio` at `/audio/{file...}`
    let app = app.route(
        "/audio/*file",
        get(audio_file_handler),
    );

async fn audio_file_handler(AxPath(file): AxPath<String>) -> axum::response::Response {
    let base = std::path::Path::new("data/audio");
    // prevent path traversal
    let safe_path = match std::path::Path::new(&file).components().filter(|c| !matches!(c, std::path::Component::ParentDir)).fold(std::path::PathBuf::new(), |mut acc, comp| { acc.push(comp); acc }) {
        p => base.join(p),
    };

    if !safe_path.exists() {
        return axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(HyperBody::from("not found"))
            .unwrap();
    }

    match fs::read(&safe_path).await {
        Ok(data) => {
            let mime = mime_guess::from_path(&safe_path).first_or_octet_stream().to_string();
            let resp = axum::response::Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .body(HyperBody::from(data))
                .unwrap();
            resp
        }
        Err(_) => axum::response::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(HyperBody::from("error opening file"))
            .unwrap(),
    }
}

// WebSocket handlers moved to `telemetry` module.

    #[cfg(feature = "tls")]
    {
        use std::path::PathBuf;
        use axum_server::tls_rustls::RustlsConfig;

        // Prefer certs in ./certs/ (mkcert workflow); fallback to plain HTTP if missing.
        let cert_path = PathBuf::from("certs/cert.pem");
        let key_path = PathBuf::from("certs/key.pem");

        if cert_path.exists() && key_path.exists() {
            info!("starting AURA-1 backend with TLS");
            let config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            let addr: SocketAddr = "0.0.0.0:8443".parse()?;
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        } else {
            let addr: SocketAddr = "0.0.0.0:8080".parse()?;
            info!("TLS certs not found; starting plain HTTP on {addr}");
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        }
    }

    #[cfg(not(feature = "tls"))]
    {
        let addr: SocketAddr = "0.0.0.0:8080".parse()?;
        info!("starting AURA-1 backend on {addr}");

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

async fn health() -> &'static str {
    "ok"
}

 

async fn telemetry_processor_task(mut rx: watch::Receiver<Pose>, telemetry_counter: Arc<AtomicU64>) {
    loop {
        // wait for a change; this ensures latest-value semantics
        if rx.changed().await.is_err() {
            // channel closed
            break;
        }
        let pose = rx.borrow().clone();
        // Simulate a small fixed-latency processing step for telemetry
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        tracing::debug!("processed pose={:?}", pose);
        let _ = telemetry_counter.load(Ordering::Relaxed);
    }
}

async fn audio_buffer_task() {
    // Placeholder for audio framing/VAD actor. In a full impl this would accept
    // a bounded channel of audio frames and perform VAD and buffering.
    let notify = Arc::new(Notify::new());
    loop {
        notify.notified().await;
        // no-op in prototype
    }
}

async fn ai_orchestrator_task(
    mut rx: mpsc::Receiver<String>,
    persist_tx: mpsc::Sender<String>,
    slow_inference: Arc<AtomicBool>,
    ai_queue_len: Arc<AtomicU64>,
) {
    while let Some(job) = rx.recv().await {
        // dequeue accounted for by enqueuer; decrease here
        ai_queue_len.fetch_sub(1, Ordering::Relaxed);
        let slow = slow_inference.load(Ordering::Relaxed);
        if slow {
            tracing::info!("AI worker: slow path enabled, sleeping");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // produce an indexing/persistence task
        let result = format!("embedding_of:{}", job);
        let _ = persist_tx.send(result).await;
    }
}

async fn persistence_worker_task(mut rx: mpsc::Receiver<String>, persist_queue_len: Arc<AtomicU64>) {
    while let Some(item) = rx.recv().await {
        // account for queue length: decrement when processing
        persist_queue_len.fetch_sub(1, Ordering::Relaxed);
        tracing::info!("persisting item (simulated): {}", item);
        // simulate RocksDB batched write latency
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[cfg(feature = "persistence")]
#[allow(dead_code)]
mod storage;

#[cfg(feature = "search")]
#[allow(dead_code)]
mod search {
    use tantivy::{
        collector::TopDocs,
        doc,
        schema::{Schema, TEXT},
        Index, IndexReader, IndexWriter, ReloadPolicy,
    };

    pub fn build_schema() -> Schema {
        let mut schema = Schema::builder();
        schema.add_text_field("content", TEXT);
        schema.build()
    }

    pub fn create_index(temp_dir: &std::path::Path) -> tantivy::Result<(IndexWriter, IndexReader)> {
        let schema = build_schema();
        let index = Index::create_in_dir(temp_dir, schema.clone())?;
        let mut writer = index.writer(50_000_000)?;
        writer.add_document(doc!(schema.get_field("content").unwrap() => "hello aura"))?;
        writer.commit()?;
            let reader = index.reader()?;
        Ok((writer, reader))
    }

    pub fn search_content(reader: &IndexReader, query_str: &str) -> tantivy::Result<Vec<String>> {
        let searcher = reader.searcher();
            let schema = searcher.index().schema();
        let content = schema.get_field("content").unwrap();
            let query = tantivy::query::QueryParser::for_index(searcher.index(), vec![content])
            .parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(5))?;
        // Return a placeholder string per hit to avoid materializing `Document` here.
        let results = vec![String::new(); top_docs.len()];
        Ok(results)
    }
}
