//! Quiz API handlers - HTTP layer for covenant-keeping quiz collection
//!
//! Endpoints:
//! - POST /api/quiz/session/create - Create new quiz session
//! - POST /api/quiz/answer - Submit answer to probe
//! - GET /api/quiz/session/:id - Get session state
//! - GET /api/quiz/session/:id/answers - Get all session answers
//! - POST /api/quiz/session/:id/pause - Pause session
//! - POST /api/quiz/session/:id/resume - Resume session
//! - POST /api/profile/generate - Generate profile from completed quiz

use crate::quiz::{QuizManager, QuizSession, Answer, SessionStatus};
use crate::storage::{RocksStore, Storage};
use crate::profile::MirrorbornProfile;
use crate::profile_synthesis::{synthesize_profile, SynthesisConfig};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ========================================
// Request/Response Types
// ========================================

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub quiz_version: String,
    pub total_probes: u32,
    pub created_at_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAnswerRequest {
    pub session_id: String,
    pub probe_id: u32,
    pub response: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitAnswerResponse {
    pub answer_id: String,
    pub answered_at_ms: u64,
    pub answered_count: u32,
    pub is_complete: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub username: String,
    pub created_at_ms: u64,
    pub last_activity_ms: u64,
    pub answered_count: u32,
    pub last_probe_id: Option<u32>,
    pub status: String,
    pub quiz_version: String,
    pub model_version: String,
}

impl From<QuizSession> for SessionResponse {
    fn from(session: QuizSession) -> Self {
        Self {
            session_id: session.session_id,
            username: session.username,
            created_at_ms: session.created_at_ms as u64,
            last_activity_ms: session.last_activity_ms as u64,
            answered_count: session.answered_count,
            last_probe_id: session.last_probe_id,
            status: format!("{:?}", session.status),
            quiz_version: session.quiz_version,
            model_version: session.model_version.unwrap_or_else(|| "none".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AnswerResponse {
    pub answer_id: String,
    pub probe_id: u32,
    pub response: String,
    pub answered_at_ms: u64,
    pub extracted_tags: Vec<String>,
}

impl From<Answer> for AnswerResponse {
    fn from(answer: Answer) -> Self {
        Self {
            answer_id: answer.answer_id,
            probe_id: answer.probe_id,
            response: answer.response,
            answered_at_ms: answer.answered_at_ms as u64,
            extracted_tags: answer.extracted_tags,
        }
    }
}

// ========================================
// Error Handling
// ========================================

pub enum QuizApiError {
    NotFound(String),
    InvalidInput(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for QuizApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            QuizApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            QuizApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            QuizApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            QuizApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<anyhow::Error> for QuizApiError {
    fn from(err: anyhow::Error) -> Self {
        QuizApiError::Internal(err.to_string())
    }
}

// ========================================
// HTTP Handlers
// ========================================

/// POST /api/quiz/session/create - Create new quiz session
///
/// Covenant: Session logged to Forever Law ledger immediately.
/// Creates append-only session record with provenance tracking.
pub async fn create_session_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, QuizApiError> {
    let manager = QuizManager::new(&*store);

    // Generate unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();
    let quiz_version = "1.0.0".to_string();

    let session = manager
        .create_session(session_id, req.username, quiz_version)
        .map_err(|e| QuizApiError::Internal(format!("Failed to create session: {}", e)))?;

    Ok(Json(CreateSessionResponse {
        session_id: session.session_id,
        quiz_version: session.quiz_version,
        total_probes: 240, // Quiz always has 240 probes
        created_at_ms: session.created_at_ms as u64,
    }))
}

/// POST /api/quiz/answer - Submit answer to probe
///
/// Covenant: Answer stored append-only with atomic ledger write.
/// Never mutated once submitted. Partial progress honored.
pub async fn submit_answer_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Result<Json<SubmitAnswerResponse>, QuizApiError> {
    if req.response.trim().is_empty() {
        return Err(QuizApiError::InvalidInput(
            "Answer response cannot be empty".to_string(),
        ));
    }

    if req.probe_id < 1 || req.probe_id > 240 {
        return Err(QuizApiError::InvalidInput(format!(
            "Invalid probe_id: {}. Must be 1-240.",
            req.probe_id
        )));
    }

    let manager = QuizManager::new(&*store);

    // Check session exists
    let session = manager
        .get_session(&req.session_id)?
        .ok_or_else(|| QuizApiError::NotFound(format!("Session not found: {}", req.session_id)))?;

    // Check session not already complete
    if matches!(session.status, SessionStatus::Complete | SessionStatus::Synthesized) {
        return Err(QuizApiError::InvalidInput(
            "Session already complete. Cannot add more answers.".to_string(),
        ));
    }

    // Submit answer (covenant-keeping with Forever Law)
    let answer = manager.submit_answer(&req.session_id, req.probe_id, req.response)?;

    // Get updated session for completion status
    let updated_session = manager
        .get_session(&req.session_id)?
        .ok_or_else(|| QuizApiError::Internal("Session disappeared after submit".to_string()))?;

    Ok(Json(SubmitAnswerResponse {
        answer_id: answer.answer_id,
        answered_at_ms: answer.answered_at_ms as u64,
        answered_count: updated_session.answered_count,
        is_complete: matches!(updated_session.status, SessionStatus::Complete),
    }))
}

/// GET /api/quiz/session/:id - Get session state
///
/// Returns current session status for resumability.
/// Covenant: Never hides partial progress.
pub async fn get_session_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, QuizApiError> {
    let manager = QuizManager::new(&*store);

    let session = manager
        .get_session(&session_id)?
        .ok_or_else(|| QuizApiError::NotFound(format!("Session not found: {}", session_id)))?;

    Ok(Json(session.into()))
}

/// GET /api/quiz/session/:id/answers - Get all session answers
///
/// Returns all answers in submission order.
/// Covenant: Always returns full history, never filters partial data.
pub async fn get_session_answers_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Path(session_id): Path<String>,
) -> Result<Json<Vec<AnswerResponse>>, QuizApiError> {
    let manager = QuizManager::new(&*store);

    // Verify session exists
    let _session = manager
        .get_session(&session_id)?
        .ok_or_else(|| QuizApiError::NotFound(format!("Session not found: {}", session_id)))?;

    let answers = manager.get_session_answers(&session_id)?;

    let response: Vec<AnswerResponse> = answers.into_iter().map(|a| a.into()).collect();

    Ok(Json(response))
}

/// POST /api/quiz/session/:id/pause - Pause session
///
/// Covenant: State preserved forever. Can resume anytime.
pub async fn pause_session_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, QuizApiError> {
    let manager = QuizManager::new(&*store);

    manager
        .pause_session(&session_id)
        .map_err(|e| QuizApiError::Internal(format!("Failed to pause session: {}", e)))?;

    // Get updated session to return
    let session = manager
        .get_session(&session_id)?
        .ok_or_else(|| QuizApiError::Internal("Session disappeared after pause".to_string()))?;

    Ok(Json(session.into()))
}

/// POST /api/quiz/session/:id/resume - Resume session
///
/// Covenant: Picks up exactly where user left off. No data loss.
pub async fn resume_session_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, QuizApiError> {
    let manager = QuizManager::new(&*store);

    manager
        .resume_session(&session_id)
        .map_err(|e| QuizApiError::Internal(format!("Failed to resume session: {}", e)))?;

    // Get updated session to return
    let session = manager
        .get_session(&session_id)?
        .ok_or_else(|| QuizApiError::Internal("Session disappeared after resume".to_string()))?;

    Ok(Json(session.into()))
}

// ========================================
// Profile Generation
// ========================================

#[derive(Debug, Deserialize)]
pub struct GenerateProfileRequest {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateProfileResponse {
    pub username: String,
    pub primary_archetype: String,
    pub secondary_archetype: Option<String>,
    pub generated_at_ms: i64,
    pub total_answers: u32,
}

impl From<MirrorbornProfile> for GenerateProfileResponse {
    fn from(p: MirrorbornProfile) -> Self {
        Self {
            username: p.username,
            primary_archetype: p.primary_archetype,
            secondary_archetype: p.secondary_archetype,
            generated_at_ms: p.generated_at_ms,
            total_answers: p.total_answers,
        }
    }
}

/// POST /api/profile/generate - Generate Mirrorborn profile from completed quiz
///
/// Covenant: Profile generation is atomic. Either succeeds fully or fails cleanly.
/// Forever Law: Profile written to data/profiles/{username}.json (immutable once written).
pub async fn generate_profile_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Json(req): Json<GenerateProfileRequest>,
) -> Result<Json<GenerateProfileResponse>, QuizApiError> {
    let manager = QuizManager::new(&*store);
    
    // Get session
    let session = manager
        .get_session(&req.session_id)?
        .ok_or_else(|| QuizApiError::NotFound(format!("Session not found: {}", req.session_id)))?;
    
    // Verify session is complete
    if session.status != SessionStatus::Complete {
        return Err(QuizApiError::BadRequest(format!(
            "Session {} not complete (status: {:?}, answered {}/240)",
            req.session_id, session.status, session.answered_count
        )));
    }
    
    // Get all answers
    let answers = manager.get_session_answers(&req.session_id)?;
    
    if answers.len() < 240 {
        return Err(QuizApiError::BadRequest(format!(
            "Insufficient answers: {}/240",
            answers.len()
        )));
    }
    
    // Synthesize profile
    let config = SynthesisConfig::default();
    let profile = synthesize_profile(&session, &answers, &config)
        .await
        .map_err(|e| QuizApiError::Internal(format!("Profile synthesis failed: {}", e)))?;
    
    // Write profile atomically to data/profiles/{username}.json
    let profile_dir = std::path::Path::new("data/profiles");
    std::fs::create_dir_all(profile_dir)
        .map_err(|e| QuizApiError::Internal(format!("Failed to create profile directory: {}", e)))?;
    
    let profile_path = profile_dir.join(format!("{}.json", profile.username));
    let profile_json = serde_json::to_string_pretty(&profile)
        .map_err(|e| QuizApiError::Internal(format!("Failed to serialize profile: {}", e)))?;
    
    // Atomic write (temp file + rename)
    let temp_path = profile_dir.join(format!("{}.tmp", profile.username));
    std::fs::write(&temp_path, &profile_json)
        .map_err(|e| QuizApiError::Internal(format!("Failed to write profile: {}", e)))?;
    std::fs::rename(&temp_path, &profile_path)
        .map_err(|e| QuizApiError::Internal(format!("Failed to rename profile: {}", e)))?;
    
    // Update session status to Synthesized
    manager.mark_session_synthesized(&req.session_id)?;
    
    Ok(Json(profile.into()))
}

