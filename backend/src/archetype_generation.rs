use axum::{extract::Extension, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use aura_orchestrator as orchestrator;

/// Request from frontend: user message → specific archetype
#[derive(Debug, Deserialize)]
pub struct ArchetypeGenerateRequest {
    pub archetype: String,
    pub user_message: String,
}

/// Response to frontend: archetype response with provenance
#[derive(Debug, Serialize)]
pub struct ArchetypeGenerateResponse {
    pub archetype: String,
    pub response: String,
    pub qsic_hash: Option<String>,
    pub article: Option<String>,
    pub temperature: f32,
}

/// Handler for /api/archetype/generate
/// Wires constitutional invoker → OllamaClient → Forever Law compliance logging
pub async fn generate_archetype_response(
    Json(payload): Json<ArchetypeGenerateRequest>,
    Extension(orchestrator_config): Extension<Arc<orchestrator::config::OrchestratorConfig>>,
    Extension(llm_client): Extension<Arc<orchestrator::llm::ollama::OllamaClient>>,
    Extension(constitutional_invoker): Extension<Arc<orchestrator::constitutional::ConstitutionalInvoker>>,
) -> Result<Json<ArchetypeGenerateResponse>, (StatusCode, String)> {
    info!(
        "Generate request: archetype={}, message_len={}",
        payload.archetype,
        payload.user_message.len()
    );

    // Map archetype string to ArchetypeId
    let archetype_id = match payload.archetype.as_str() {
        "Sentinel" => orchestrator::prompts::ArchetypeId::Sentinel,
        "Architect" => orchestrator::prompts::ArchetypeId::Architect,
        "Explorer" => orchestrator::prompts::ArchetypeId::Explorer,
        "Oracle" => orchestrator::prompts::ArchetypeId::Oracle,
        "Mentor" => orchestrator::prompts::ArchetypeId::Mentor,
        "Empath" => orchestrator::prompts::ArchetypeId::Empath,
        "Jester" => orchestrator::prompts::ArchetypeId::Jester,
        unknown => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown archetype: {}", unknown),
            ))
        }
    };

    // Get archetype config (model, temperature, top_p)
    let archetype_config = orchestrator_config
        .archetypes
        .get(&payload.archetype)
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Archetype config missing: {}", payload.archetype),
            )
        })?;

    let params = orchestrator::llm::GenerationParams {
        model: archetype_config.model.clone(),
        temperature: archetype_config.temperature as f32,
        top_p: archetype_config.top_p as f32,
    };

    // Invoke through constitutional layer
    let result = constitutional_invoker
        .generate(archetype_id.clone(), &payload.user_message, &params, llm_client.as_ref())
        .await
        .map_err(|e| {
            error!("Constitutional generation failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Generation failed: {:?}", e),
            )
        })?;

    // TODO: Parse DECISION/ARTICLE/REASON format for Sentinel
    // TODO: Generate QSIC provenance hash
    // TODO: Log Forever Law compliance (article citations, user context, response)

    info!(
        "Generated response for {}: {} chars",
        payload.archetype,
        result.text.len()
    );

    Ok(Json(ArchetypeGenerateResponse {
        archetype: payload.archetype,
        response: result.text,
        qsic_hash: None, // TODO: QSIC integration
        article: None,   // TODO: Parse Sentinel verdict
        temperature: params.temperature,
    }))
}
