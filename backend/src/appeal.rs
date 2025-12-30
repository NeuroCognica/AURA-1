use axum::{extract::Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::storage;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppealRequest {
    pub session_id: String,
    pub user_id: Option<String>,
    pub reason: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AppealResponse {
    pub id: u64,
}

// Accepts an appeal POST and records it in the append-only log if persistence is enabled.
// When persistence is not enabled, returns 501 Not Implemented.
pub async fn appeal_handler(
    Json(req): Json<AppealRequest>,
    Extension(store): Extension<Option<Arc<storage::RocksStore>>>,
) -> (StatusCode, String) {
    if let Some(store) = store {
        // Serialize the appeal into JSON for storage; fallback to reason if serialization fails.
        let content = serde_json::to_string(&req).unwrap_or_else(|_| req.reason.clone());
        match store.append_log_atomic("appeal", &content) {
            Ok(id) => (StatusCode::CREATED, serde_json::json!({"id": id}).to_string()),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
        }
    } else {
        (StatusCode::NOT_IMPLEMENTED, "persistence not enabled".to_string())
    }
}
