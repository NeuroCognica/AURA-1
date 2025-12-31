use crate::council_verdict::*;
use crate::storage::RocksStore;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppealError {
    #[error("no active appeal")]
    NoActiveAppeal,
    #[error("verdict mismatch")]
    VerdictMismatch,
    #[error("consent phrase mismatch")]
    ConsentMismatch,
    #[error("override token expired")]
    TokenExpired,
    #[error("hard deny cannot be overridden")]
    HardDeny,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("serialization error: {0}")]
    Ser(String),
}

fn k_state(sid: &str) -> String {
    format!("session:{}:appeal_state", sid)
}
fn k_verdict_last(sid: &str) -> String {
    format!("session:{}:verdict:last", sid)
}
fn k_verdict(sid: &str, vid: &str) -> String {
    format!("session:{}:verdict:{}", sid, vid)
}

pub fn load_appeal_state(store: &RocksStore, sid: &str) -> Result<AppealState, AppealError> {
    match store
        .get_bytes(k_state(sid).as_bytes())
        .map_err(|e| AppealError::Storage(e.to_string()))?
    {
        None => Ok(AppealState::Idle),
        Some(b) => serde_json::from_slice(&b).map_err(|e| AppealError::Ser(e.to_string())),
    }
}

pub fn save_appeal_state(
    store: &RocksStore,
    sid: &str,
    st: &AppealState,
) -> Result<(), AppealError> {
    let b = serde_json::to_vec(st).map_err(|e| AppealError::Ser(e.to_string()))?;
    store
        .put_bytes(k_state(sid).as_bytes(), &b)
        .map_err(|e| AppealError::Storage(e.to_string()))?;
    Ok(())
}

pub fn save_verdict(store: &RocksStore, sid: &str, v: &CouncilVerdict) -> Result<(), AppealError> {
    let b = serde_json::to_vec(v).map_err(|e| AppealError::Ser(e.to_string()))?;
    store
        .put_bytes(k_verdict(sid, &v.verdict_id.0).as_bytes(), &b)
        .map_err(|e| AppealError::Storage(e.to_string()))?;
    store
        .put_bytes(k_verdict_last(sid).as_bytes(), v.verdict_id.0.as_bytes())
        .map_err(|e| AppealError::Storage(e.to_string()))?;
    Ok(())
}

pub fn load_verdict(
    store: &RocksStore,
    sid: &str,
    vid: &str,
) -> Result<CouncilVerdict, AppealError> {
    let b = store
        .get_bytes(k_verdict(sid, vid).as_bytes())
        .map_err(|e| AppealError::Storage(e.to_string()))?
        .ok_or_else(|| AppealError::NoActiveAppeal)?;
    serde_json::from_slice(&b).map_err(|e| AppealError::Ser(e.to_string()))
}

pub fn on_verdict(_st: AppealState, v: &CouncilVerdict) -> AppealState {
    match v.final_state {
        FinalState::Allowed | FinalState::AllowedWithWarning => AppealState::Closed {
            verdict_id: v.verdict_id.clone(),
            final_state: v.final_state.clone(),
        },
        FinalState::RequireConsent | FinalState::RequireUserChoice | FinalState::Blocked => {
            AppealState::AwaitingUser {
                verdict_id: v.verdict_id.clone(),
                required: v.next_required.clone(),
            }
        }
    }
}

pub fn submit_consent(
    st: &AppealState,
    v: &CouncilVerdict,
    phrase: &str,
    now_ms: u128,
) -> Result<AppealState, AppealError> {
    let vid = match st {
        AppealState::AwaitingUser { verdict_id, .. } => verdict_id.clone(),
        _ => return Err(AppealError::NoActiveAppeal),
    };
    if vid != v.verdict_id {
        return Err(AppealError::VerdictMismatch);
    }
    if let SentinelDecisionKind::Deny = v.sentinel_kind {
        return Err(AppealError::HardDeny);
    }

    let c = v.consent.as_ref().ok_or(AppealError::NoActiveAppeal)?;
    if phrase != c.exact_phrase {
        return Err(AppealError::ConsentMismatch);
    }

    let token = OverrideToken {
        token_id: format!("tok_{}", now_ms),
        scope: c.scope.clone(),
        issued_at_ms: now_ms,
        ttl_seconds: c.ttl_seconds,
    };
    let expires_at_ms = now_ms + (c.ttl_seconds as u128) * 1000;
    Ok(AppealState::Authorized {
        verdict_id: v.verdict_id.clone(),
        token,
        expires_at_ms,
    })
}

pub fn token_valid(st: &AppealState, now_ms: u128) -> Result<(), AppealError> {
    match st {
        AppealState::Authorized { expires_at_ms, .. } if now_ms <= *expires_at_ms => Ok(()),
        AppealState::Authorized { .. } => Err(AppealError::TokenExpired),
        _ => Ok(()),
    }
}

pub fn ws_msg_verdict(v: &CouncilVerdict) -> CouncilWsMsg {
    CouncilWsMsg {
        kind: "verdict".to_string(),
        session_id: v.session_id.0.clone(),
        payload: serde_json::to_value(v).unwrap_or_else(|_| json!({"error":"serialize"})),
    }
}

pub fn ws_msg_state(sid: &str, st: &AppealState) -> CouncilWsMsg {
    CouncilWsMsg {
        kind: "appeal_state".to_string(),
        session_id: sid.to_string(),
        payload: serde_json::to_value(st).unwrap_or_else(|_| json!({"error":"serialize"})),
    }
}
use axum::http::StatusCode;
use axum::{extract::Extension, Json};
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
            Ok(id) => (
                StatusCode::CREATED,
                serde_json::json!({"id": id}).to_string(),
            ),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("error: {}", e)),
        }
    } else {
        (
            StatusCode::NOT_IMPLEMENTED,
            "persistence not enabled".to_string(),
        )
    }
}
