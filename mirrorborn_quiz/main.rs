use axum::{extract::Extension, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

mod store;
mod quiz;
mod mirrorborn;

use store::{AnswerRecord, Store};
use quiz::TOTAL_PROBES;
use mirrorborn::generate_profile;

#[derive(Debug, Deserialize)]
struct SubmitAnswer {
    session_id: String,
    probe_id: String,
    answer: String,
}

#[derive(Debug, Serialize)]
struct SubmitResponse {
    success: bool,
    answered_count: u64,
}

async fn submit_handler(
    Json(payload): Json<SubmitAnswer>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<SubmitResponse> {
    let idx = match state.store.upsert_answer(&payload.session_id, &payload.probe_id, &payload.answer) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("upsert error: {}", e);
            0
        }
    };

    let cnt = state.store.answered_count(&payload.session_id).unwrap_or(0);

    // If complete, spawn generator
    if (cnt as usize) >= TOTAL_PROBES {
        let s = state.clone();
        let sid = payload.session_id.clone();
        tokio::spawn(async move {
            match generate_profile(&s.store, &sid, &s.artifacts_dir) {
                Ok(p) => eprintln!("Generated profile: {}", p.display()),
                Err(e) => eprintln!("Profile generation failed: {}", e),
            }
        });
    }

    Json(SubmitResponse { success: true, answered_count: cnt })
}

async fn session_handler(
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let answers = match state.store.get_session_answers(&session_id) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("get_session_answers: {}", e);
            vec![]
        }
    };
    let cnt = state.store.answered_count(&session_id).unwrap_or(0);
    Json(serde_json::json!({ "session_id": session_id, "answered_count": cnt, "answers": answers }))
}

async fn generate_handler(
    axum::extract::Path(session_id): axum::extract::Path<String>,
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    match generate_profile(&state.store, &session_id, &state.artifacts_dir) {
        Ok(p) => Ok(Json(serde_json::json!({ "path": p.display().to_string() }))),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

#[derive(Clone)]
struct AppState {
    store: Store,
    artifacts_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("QUIZ_DB_PATH").unwrap_or_else(|_| "./quiz_db".to_string());
    let artifacts_dir = std::env::var("ARTIFACTS_DIR").unwrap_or_else(|_| "./artifacts".to_string());

    let store = Store::open(db_path).expect("open store");

    let state = Arc::new(AppState { store, artifacts_dir: PathBuf::from(artifacts_dir) });

    let app = Router::new()
        .route("/submit_answer", post(submit_handler))
        .route("/session/:session_id", get(session_handler))
        .route("/generate/:session_id", post(generate_handler))
        .layer(Extension(state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4001));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
