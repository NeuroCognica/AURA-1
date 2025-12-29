use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let app = Router::new()
        .route("/health", get(health))
        .route("/ws/telemetry", get(ws_handler));

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    info!("starting AURA-1 backend on {addr}");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, addr))
}

async fn handle_ws(mut socket: WebSocket, addr: SocketAddr) {
    info!("telemetry ws connected: {addr}");
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if text.len() > 4_096 {
                    warn!("telemetry text too large from {addr}, dropping");
                    continue;
                }
                info!("telemetry text from {addr}: {text}");
            }
            Ok(Message::Binary(bytes)) => {
                info!("telemetry binary from {addr}: {} bytes", bytes.len());
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => {
                info!("telemetry ws closing: {addr}");
                break;
            }
            Err(err) => {
                warn!("telemetry ws error from {addr}: {err}");
                break;
            }
        }
    }
}

#[cfg(feature = "persistence")]
#[allow(dead_code)]
mod storage {
    use blake3::Hash;
    use merklemountainrange::{Hash as MmrHash, Mmr};
    use rocksdb::{Options, DB};
    use std::path::Path;

    pub struct AppendOnlyStore {
        db: DB,
    }

    impl AppendOnlyStore {
        pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
            let mut opts = Options::default();
            opts.create_if_missing(true);
            let db = DB::open(&opts, path)?;
            Ok(Self { db })
        }

        pub fn append_log(&self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
            self.db.put(key, value)?;
            Ok(())
        }
    }

    pub struct RocksMmr {
        inner: Mmr<Vec<u8>, Hash>,
    }

    impl RocksMmr {
        pub fn new() -> Self {
            Self {
                inner: Mmr::new(Vec::new()),
            }
        }

        pub fn push(&mut self, leaf: &[u8]) -> anyhow::Result<Hash> {
            let digest = blake3::hash(leaf);
            self.inner.push(digest.as_bytes().to_vec())?;
            Ok(digest)
        }
    }
}

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
        let reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;
        Ok((writer, reader))
    }

    pub fn search_content(reader: &IndexReader, query_str: &str) -> tantivy::Result<Vec<String>> {
        let searcher = reader.searcher();
        let schema = reader.index().schema();
        let content = schema.get_field("content").unwrap();
        let query = tantivy::query::QueryParser::for_index(reader.index(), vec![content])
            .parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(5))?;
        let mut results = Vec::new();
        for (_score, doc_addr) in top_docs {
            let retrieved = searcher.doc(doc_addr)?;
            if let Some(val) = retrieved.get_first(content) {
                results.push(val.text().unwrap_or_default().to_string());
            }
        }
        Ok(results)
    }
}
