/// HTTP API endpoints for account management

use axum::{extract::Extension, http::StatusCode, Json};
use serde_json::json;
use std::sync::Arc;

#[cfg(feature = "persistence")]
use crate::accounts::{AccountManager, CreateAccountRequest, LoginRequest, LoginResponse};
#[cfg(feature = "persistence")]
use crate::storage::RocksStore;

/// POST /api/account/create - Create new account
#[cfg(feature = "persistence")]
#[axum::debug_handler]
pub async fn create_account_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Json(req): Json<CreateAccountRequest>,
) -> impl axum::response::IntoResponse {
    let mgr = AccountManager::new(&*store);

    match mgr.create_account(&req.username, &req.password) {
        Ok(account) => {
            tracing::info!("Account created via API: {}", account.username);
            (
                StatusCode::CREATED,
                Json(json!({
                    "success": true,
                    "username": account.username,
                    "created_at_ms": account.created_at_ms
                })),
            )
        }
        Err(e) => {
            tracing::warn!("Account creation failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": e.to_string()
                })),
            )
        }
    }
}

/// POST /api/account/login - Authenticate user
#[cfg(feature = "persistence")]
#[axum::debug_handler]
pub async fn login_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Json(req): Json<LoginRequest>,
) -> impl axum::response::IntoResponse {
    let mgr = AccountManager::new(&*store);

    match mgr.authenticate(&req.username, &req.password) {
        Ok(Some(account)) => {
            tracing::info!("Login via API: {}", account.username);
            (
                StatusCode::OK,
                Json(LoginResponse {
                    success: true,
                    username: Some(account.username),
                    error: None,
                }),
            )
        }
        Ok(None) => {
            tracing::warn!("Login failed via API: {}", req.username);
            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    success: false,
                    username: None,
                    error: Some("Invalid username or password".to_string()),
                }),
            )
        }
        Err(e) => {
            tracing::error!("Login error via API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    success: false,
                    username: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            )
        }
    }
}

/// GET /api/account/:username - Get account info (for logged-in user)
#[cfg(feature = "persistence")]
#[axum::debug_handler]
pub async fn get_account_handler(
    axum::extract::Path(username): axum::extract::Path<String>,
    Extension(store): Extension<Arc<RocksStore>>,
) -> impl axum::response::IntoResponse {
    let mgr = AccountManager::new(&*store);

    match mgr.get_account(&username) {
        Ok(Some(account)) => (
            StatusCode::OK,
            Json(json!({
                "username": account.username,
                "created_at_ms": account.created_at_ms,
                "last_login_ms": account.last_login_ms,
                "quiz_sessions": account.quiz_sessions
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Account not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}
