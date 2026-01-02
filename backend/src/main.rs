use axum::body::Body as HyperBody;
use axum::extract::Path as AxPath;
use axum::http::{header, StatusCode};
use axum::{extract::Extension, routing::{get, post}, Json, Router};
use base64::Engine;
use mime_guess;
use rustls::crypto::ring;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch, Notify};
use tracing::{info, warn};


mod telemetry;
use crate::telemetry::Pose;
mod broadcast;
use crate::broadcast as broadcast_mod;
mod appeal;
mod council_verdict;
mod storage;
mod intent_stratification;
mod archetype_api;
mod generation_manager;
mod chat_api;
#[cfg(feature = "persistence")]
mod ollama;
mod sentinel;
#[cfg(feature = "persistence")]
mod accounts;
#[cfg(feature = "persistence")]
mod accounts_api;
use crate::generation_manager as generation_manager_mod;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    // Ensure rustls has a concrete crypto provider installed for this process.
    ring::default_provider()
        .install_default()
        .expect("failed to install rustls ring crypto provider");

    // latest-value telemetry path (watch channel)
    let (pose_tx, _pose_rx) = watch::channel(Pose {
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
    });

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
    let (ai_bcast_tx, _ai_bcast_rx) = tokio::sync::broadcast::channel::<String>(1024);
    let (council_bcast_tx, _council_bcast_rx) = tokio::sync::broadcast::channel::<String>(256);
    // Typed council broadcast channel (migration path to typed transport)
    let (council_bcast_typed_tx, _council_bcast_typed_rx) =
        tokio::sync::broadcast::channel::<crate::council_verdict::CouncilEnvelope>(256);

    // GenerationManager for cancellation / gen_id tracking
    let gen_mgr = std::sync::Arc::new(generation_manager_mod::GenerationManager::new());

    // Load archetypes JSON profiles from repository root `../archetypes` (relative to backend/)
    let archetypes = match load_archetypes("../archetypes") {
        Ok(m) => std::sync::Arc::new(m),
        Err(e) => {
            warn!("failed to load archetypes: {}", e);
            std::sync::Arc::new(HashMap::new())
        }
    };

    // Spawn actors
    tokio::spawn(telemetry_processor_task(
        pose_tx.subscribe(),
        telemetry_counter.clone(),
    ));
    tokio::spawn(audio_buffer_task());
    tokio::spawn(ai_orchestrator_task(
        ai_rx,
        persist_tx.clone(),
        slow_inference.clone(),
        ai_queue_len.clone(),
    ));
    tokio::spawn(persistence_worker_task(
        persist_rx,
        persist_queue_len.clone(),
    ));

    #[cfg(feature = "persistence")]
    let store = {
        use std::path::PathBuf;
        let path = PathBuf::from("data/rocksdb");
        std::fs::create_dir_all(&path)?;
        let s = storage::RocksStore::open(path)?;
        Some(std::sync::Arc::new(s))
    };

    // Startup check: verify configured Ollama model is reachable. Non-fatal: warn only.
    {
        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        let model = std::env::var("OLLAMA_DEFAULT_MODEL")
            .unwrap_or_else(|_| "deepseek-coder:6.7b".to_string());
        let url = ollama_url.clone();
        let model_name = model.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let list_url = format!("{}/api/models", url.trim_end_matches('/'));
            match client.get(&list_url).send().await {
                Ok(r) => match r.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // Expect an array or object; do a simple substring search for model name
                        let s = json.to_string();
                        if !s.contains(&model_name) {
                            warn!(
                                "configured ollama model not found: {} (models response: {})",
                                model_name, s
                            );
                        } else {
                            info!("ollama model present: {}", model_name);
                        }
                    }
                    Err(e) => warn!(%e, "failed to parse /api/models response from Ollama"),
                },
                Err(e) => warn!(%e, "failed to reach Ollama at {}", url),
            }
        });
    }

    // Optional synthetic telemetry generator (env var: SYNTHETIC_TELEMETRY=1)
    if std::env::var("SYNTHETIC_TELEMETRY").as_deref() == Ok("1") {
        info!("starting synthetic 60Hz telemetry generator");
        let gen_tx = pose_tx.clone();
        tokio::spawn(async move {
            let mut t = tokio::time::interval(std::time::Duration::from_micros(16_667)); // ~60Hz
            loop {
                t.tick().await;
                let now = Instant::now();
                let pose = Pose {
                    yaw: now.elapsed().as_secs_f32() % 1.0,
                    pitch: 0.0,
                    roll: 0.0,
                };
                let _ = gen_tx.send(pose);
            }
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/metrics",
            get({
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
            }),
        )
        .route("/ws/telemetry", get(telemetry::ws_telemetry_handler))
        // Ingest endpoints (PC side) — connectors for native sidecars
        .route("/ws/pose-ingest", get(broadcast_mod::ws_pose_ingest))
        .route("/ws/voice-ingest", get(broadcast_mod::ws_voice_ingest))
        // Client subscription endpoints (iPhone viewport)
        .route("/ws/pose", get(broadcast_mod::ws_pose_client))
        .route("/ws/voice", get(broadcast_mod::ws_voice_client))
        .route("/ws/ai", get(broadcast_mod::ws_ai_handler))
        .route("/ws/council", get(broadcast_mod::ws_council_handler));
    
    // Account management endpoints (offline login system)
    #[cfg(feature = "persistence")]
    let app = app
        .route("/api/account/create", post(accounts_api::create_account_handler))
        .route("/api/account/login", post(accounts_api::login_handler))
        .route("/api/account/:username", get(accounts_api::get_account_handler));

    // Quiz collection endpoints (covenant-keeping 240-question profiling)
    #[cfg(feature = "persistence")]
    let app = {
        use aura_backend::quiz_api;
        app
            .route("/api/quiz/session/create", post(quiz_api::create_session_handler))
            .route("/api/quiz/answer", post(quiz_api::submit_answer_handler))
            .route("/api/quiz/session/:id", get(quiz_api::get_session_handler))
            .route("/api/quiz/session/:id/answers", get(quiz_api::get_session_answers_handler))
            .route("/api/quiz/session/:id/pause", post(quiz_api::pause_session_handler))
            .route("/api/quiz/session/:id/resume", post(quiz_api::resume_session_handler))
    };

    let app = app
        .route(
            "/api/archetypes",
            get({
                let arche = archetypes.clone();
                move || {
                    let arche = arche.clone();
                    async move {
                        // clone the map for response serialization
                        let data = arche.as_ref().clone();
                        (axum::http::StatusCode::OK, Json(data))
                    }
                }
            }),
        )
        // Debug: read recent session messages (persistence feature only)
        .route(
            "/debug/session/:id",
            get(|AxPath(session_id): AxPath<String>| async move {
                // Open RocksDB on demand for debugging so we don't depend on router extensions here.
                match storage::RocksStore::open(std::path::PathBuf::from("data/rocksdb")) {
                    Ok(store) => match store.load_recent_chat(&session_id, 200) {
                        Ok(v) => (
                            StatusCode::OK,
                            serde_json::to_string(&v).unwrap_or_else(|_| "[]".to_string()),
                        ),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                    },
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("error opening db: {}", e),
                    ),
                }
            }),
        )
        .route(
            "/toggle_slow_inference",
            get({
                let slow_inference = slow_inference.clone();
                move || async move {
                    let prev = slow_inference.fetch_xor(true, Ordering::SeqCst);
                    let state = if prev { "off" } else { "on" };
                    (
                        axum::http::StatusCode::OK,
                        format!("slow_inference {}", state),
                    )
                }
            }),
        )
        .route(
            "/enqueue_ai",
            axum::routing::post({
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
            }),
        )
        .route("/", get(|| async { "AURA-1 backend prototype" }));

    // Expose archetype activation route using the existing library handler and
    // provide the required Extensions (archetypes map, council broadcasts).
    let app = app
        .route(
            "/api/archetype/activate",
            axum::routing::post({
                let arche = archetypes.clone();
                let council = council_bcast_tx.clone();
                let council_typed = council_bcast_typed_tx.clone();
                move |axum::Json(payload): axum::Json<serde_json::Value>| {
                    let arche = arche.clone();
                    let council = council.clone();
                    let council_typed = council_typed.clone();
                    async move {
                        crate::archetype_api::activate_archetype_handler(
                            axum::Json(payload),
                            axum::Extension(arche),
                            axum::Extension(council),
                            axum::Extension(council_typed),
                        )
                        .await
                    }
                }
            }),
        )
        .layer(Extension(archetypes.clone()))
        .layer(Extension(council_bcast_tx.clone()))
        .layer(Extension(council_bcast_typed_tx.clone()));

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
                                let resp = serde_json::json!({"leaf": base64::engine::general_purpose::STANDARD.encode(leaf), "peaks": peaks});
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
                .route("/api/chat", axum::routing::post(crate::chat_api::chat_handler))
                .route("/api/appeal", axum::routing::post({
                    let store = store.clone();
                    move |Json(payload): Json<crate::appeal::AppealRequest>| {
                        let store = store.clone();
                        async move {
                            let content = serde_json::to_string(&payload).unwrap_or_else(|_| payload.reason.clone());
                            match store.append_log_atomic("appeal", &content) {
                                Ok(id) => (axum::http::StatusCode::CREATED, serde_json::json!({"id": id}).to_string()),
                                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                            }
                        }
                    }
                }))
                .route("/api/appeal/consent", axum::routing::post({
                    let store = store.clone();
                    let ai_bcast = ai_bcast_tx.clone();
                    let council_bcast = council_bcast_tx.clone();
                    let council_bcast_typed = council_bcast_typed_tx.clone();
                    let gen_mgr = gen_mgr.clone();
                    move |Json(req): Json<crate::council_verdict::ConsentSubmitRequest>| {
                        let store = store.clone();
                        let ai_bcast = ai_bcast.clone();
                        let council_bcast = council_bcast.clone();
                        let gen_mgr = gen_mgr.clone();
                        async move {
                            // load verdict
                            match crate::appeal::load_verdict(&*store, &req.session_id, &req.verdict_id) {
                                Ok(v) => {
                                    // load current state
                                        let st = match crate::appeal::load_appeal_state(&*store, &req.session_id) {
                                        Ok(s) => s,
                                        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                                    };

                                    let now_ms = chrono::Utc::now().timestamp_millis() as u128;
                                    match crate::appeal::submit_consent(&st, &v, &req.phrase, now_ms) {
                                        Ok(new_st) => {
                                            // persist and broadcast state
                                            if let Err(e) = crate::appeal::save_appeal_state(&*store, &req.session_id, &new_st) {
                                                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("save error: {}", e));
                                            }
                                            // Persist and broadcast the new appeal state on the council channel
                                            let msg = crate::appeal::ws_msg_state(&req.session_id, &new_st);
                                            crate::broadcast::broadcast_council(&store, &council_bcast, Some(&council_bcast_typed), &req.session_id, "appeal_state", serde_json::to_value(&msg.payload).unwrap_or_else(|_| serde_json::json!({})), Some(&gen_mgr)); 
                                            let resp = serde_json::json!({"ok": true, "state": new_st});
                                            return (axum::http::StatusCode::OK, resp.to_string());
                                        }
                                        Err(e) => {
                                            return (axum::http::StatusCode::BAD_REQUEST, format!("error: {}", e));
                                        }
                                    }
                                }
                                Err(e) => return (axum::http::StatusCode::NOT_FOUND, format!("verdict load error: {}", e)),
                            }
                        }
                    }
                }))
                .route("/api/appeal/alchemist", axum::routing::post({
                    let store = store.clone();
                    move |Json(req): Json<crate::council_verdict::AlchemistInvokeRequest>| {
                        let store = store.clone();
                        async move {
                            // stub: record attempt and return not implemented
                            let body = serde_json::to_string(&req).unwrap_or_default();
                            let _ = store.append_chat_msg(&req.session_id, "appeal_alchemist", &body);
                            (axum::http::StatusCode::NOT_IMPLEMENTED, serde_json::json!({"ok": false, "reason": "alchemist not implemented"}).to_string())
                        }
                    }
                }))
                // Session mode API: GET latest session meta and POST mode changes (append-only + audit log)
                .route("/api/session/:id/mode", axum::routing::get({
                    let store = store.clone();
                    move |AxPath(session_id): AxPath<String>| {
                        let store = store.clone();
                        async move {
                            match store.load_recent_session_meta(&session_id, 1) {
                                Ok(mut v) => {
                                    if v.is_empty() {
                                        (axum::http::StatusCode::OK, "{}".to_string())
                                    } else {
                                        let last = v.pop().unwrap();
                                        (axum::http::StatusCode::OK, serde_json::to_string(&last).unwrap_or_else(|_| "{}".to_string()))
                                    }
                                }
                                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                            }
                        }
                    }
                }))
                .route("/api/session/:id/mode", axum::routing::post({
                    let store = store.clone();
                    move |AxPath(session_id): AxPath<String>, Json(payload): Json<serde_json::Value>| {
                        let store = store.clone();
                        async move {
                            // accept arbitrary JSON metadata; write append-only and also append an audit chat message
                            let content = match serde_json::to_string(&payload) {
                                Ok(s) => s,
                                Err(e) => return (axum::http::StatusCode::BAD_REQUEST, format!("json error: {}", e)),
                            };

                            match store.append_session_meta(&session_id, &content) {
                                Ok(_) => {
                                    // also append an audit entry to the session chat
                                    let audit_msg = format!("mode_change: {}", content);
                                    let _ = store.append_chat_msg(&session_id, "system", &audit_msg);
                                    (axum::http::StatusCode::CREATED, "{}".to_string())
                                }
                                Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                            }
                        }
                    }
                }))
            .layer(Extension(store.clone()))
    };

    // Attach shared state as axum `Extension`s so handlers can extract them.
    let app = app
        .layer(Extension(pose_tx.clone()))
        .layer(Extension(telemetry_counter.clone()))
        .layer(Extension(pose_bcast_tx.clone()))
        .layer(Extension(voice_bcast_tx.clone()));
    let app = app.layer(Extension(ai_bcast_tx.clone()));
    let app = app.layer(Extension(council_bcast_tx.clone()));
    // Expose the typed council broadcast channel so handlers can publish typed envelopes.
    let app = app.layer(Extension(council_bcast_typed_tx.clone()));
    let app = app.layer(Extension(gen_mgr.clone()));

    // Serve static TTS/audio files from `backend/data/audio` at `/audio/{file...}`
    let app = app.route("/audio/*file", get(audio_file_handler));

    // Debug: route that extracts the in-memory `store` Extension and returns recent chat
    let app = app.route(
        "/debug/session_ext/:id",
        get(
            |AxPath(session_id): AxPath<String>,
             Extension(store): Extension<Option<Arc<storage::RocksStore>>>| async move {
                if let Some(store) = store {
                    match store.load_recent_chat(&session_id, 200) {
                        Ok(v) => (
                            StatusCode::OK,
                            serde_json::to_string(&v).unwrap_or_else(|_| "[]".to_string()),
                        ),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
                    }
                } else {
                    (StatusCode::NOT_FOUND, "persistence not enabled".to_string())
                }
            },
        ),
    );

    async fn audio_file_handler(AxPath(file): AxPath<String>) -> axum::response::Response {
        let base = std::path::Path::new("data/audio");
        // prevent path traversal
        let safe_path = match std::path::Path::new(&file)
            .components()
            .filter(|c| !matches!(c, std::path::Component::ParentDir))
            .fold(std::path::PathBuf::new(), |mut acc, comp| {
                acc.push(comp);
                acc
            }) {
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
                let mime = mime_guess::from_path(&safe_path)
                    .first_or_octet_stream()
                    .to_string();
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
        use axum_server::tls_rustls::RustlsConfig;
        use std::path::PathBuf;

        // Prefer certs in ./certs/ (mkcert workflow); fallback to plain HTTP if missing.
        let cert_path = PathBuf::from("certs/cert.pem");
        let key_path = PathBuf::from("certs/key.pem");

        if cert_path.exists() && key_path.exists() {
            info!("starting AURA-1 backend with TLS");
            info!("AURA backend starting -- binding HTTP server");
            let config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            let addr: SocketAddr = "0.0.0.0:8443".parse()?;
            info!("AURA backend listening on {}", addr);
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        } else {
            let addr: SocketAddr = "0.0.0.0:8080".parse()?;
            info!("TLS certs not found; starting plain HTTP on {addr}");
            info!("AURA backend starting -- binding HTTP server");
            let listener = TcpListener::bind(addr).await?;
            info!("AURA backend listening on {}", addr);
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await?;
        }
    }

    #[cfg(not(feature = "tls"))]
    {
        let addr: SocketAddr = "0.0.0.0:8080".parse()?;
        info!("starting AURA-1 backend on {addr}");
        info!("AURA backend starting -- binding HTTP server");

        let listener = TcpListener::bind(addr).await?;
        info!("AURA backend listening on {}", addr);
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;
    }
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

fn load_archetypes(dir: &str) -> anyhow::Result<HashMap<String, Value>> {
    let mut map: HashMap<String, Value> = HashMap::new();
    let p = std::path::Path::new(dir);
    if !p.exists() {
        return Ok(map);
    }

    for entry in std::fs::read_dir(p)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let s = std::fs::read_to_string(&path)?;
        let v: Value = serde_json::from_str(&s)?;
        let key = v
            .get("id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                path.file_stem()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        map.insert(key, v);
    }
    Ok(map)
}

async fn health() -> &'static str {
    "ok"
}

// Reusable handler for archetype activation. Extracted so tests can call it.
pub async fn activate_archetype_handler(
    axum::Json(payload): axum::Json<Value>,
    Extension(arche): Extension<std::sync::Arc<std::collections::HashMap<String, Value>>>,
    Extension(council_bcast): Extension<tokio::sync::broadcast::Sender<String>>,
    Extension(council_bcast_typed): Extension<tokio::sync::broadcast::Sender<crate::council_verdict::CouncilEnvelope>>,
) -> (axum::http::StatusCode, axum::Json<Value>) {
    let archetype_str = payload
        .get("archetype")
        .and_then(|v| v.as_str())
        .unwrap_or("architect");
    let ritual = payload.get("ritual").and_then(|v| v.as_bool()).unwrap_or(false);

    let key = archetype_str.to_string();
    let theme_vars = arche
        .get(&key)
        .and_then(|v| v.get("cssVars"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let audio_sig = arche
        .get(&key)
        .and_then(|v| v.get("audio"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"tone":"F#","duration_ms":400,"wave":"sine"}));

    let transition = if ritual {
        serde_json::json!({"mode": "ritual", "duration_ms": 600})
    } else {
        serde_json::json!({"mode": "instant", "duration_ms": 0})
    };

    let resp = serde_json::json!({
        "archetype": archetype_str,
        "theme_vars": theme_vars,
        "audio_signature": audio_sig,
        "transition": transition
    });

    let msg = crate::council_verdict::CouncilWsMsg {
        kind: "archetype_changed".to_string(),
        session_id: "system".to_string(),
        payload: resp.clone(),
    };

    if let Ok(s) = serde_json::to_string(&msg) {
        let _ = council_bcast.send(s.clone());
    }

    // typed envelope publish (best-effort)
    let env = crate::council_verdict::CouncilEnvelope {
        seq: 0,
        msg_type: crate::council_verdict::CouncilMsgType::SentinelNotice,
        ts_ms: crate::council_verdict::now_ms(),
        sid: "system".to_string(),
        vid: None,
        payload: resp.clone(),
    };
    let _ = council_bcast_typed.send(env);

    (axum::http::StatusCode::OK, axum::Json(resp))
}

async fn telemetry_processor_task(
    mut rx: watch::Receiver<Pose>,
    telemetry_counter: Arc<AtomicU64>,
) {
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

async fn persistence_worker_task(
    mut rx: mpsc::Receiver<String>,
    persist_queue_len: Arc<AtomicU64>,
) {
    while let Some(item) = rx.recv().await {
        // account for queue length: decrement when processing
        persist_queue_len.fetch_sub(1, Ordering::Relaxed);
        tracing::info!("persisting item (simulated): {}", item);
        // simulate RocksDB batched write latency
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

// `storage` module is declared at crate root (backend/src/storage.rs)
// storage implementation is feature-gated inside the module file itself.

#[cfg(feature = "search")]
#[allow(dead_code)]
mod search {
    use tantivy::{
        collector::TopDocs,
        doc,
        schema::{Schema, TEXT},
        Index, IndexReader, IndexWriter,
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
