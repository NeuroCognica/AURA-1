use axum::{extract::Json, Extension};
use axum::http::StatusCode;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono;
use std::sync::Arc;
use crate::storage::RocksStore;
use crate::sentinel::{sentinel_evaluate, sentinel_speak, format_sentinel_block, SentinelDecision};
use crate::council_verdict::CouncilWsMsg;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub role: String,
    pub content: String,
    pub ts: i64,
}

#[derive(Serialize)]
struct OllamaGenerateReq {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaGenerateOptions>,
}

#[derive(Serialize)]
struct OllamaGenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_gpu: Option<u32>,
}

#[derive(Serialize)]
struct OllamaMsg {
    role: String,
    content: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub ollama_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub retrieval: Option<String>,
}

#[derive(Deserialize)]
struct OllamaChatStreamChunk {
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    done: bool,
}

pub type BoxedTextStream = Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send>>;

#[derive(Debug)]
enum SessionMode {
    Code,
    Planning,
    Voice,
    Unknown,
}

impl SessionMode {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "code" => SessionMode::Code,
            "planning" => SessionMode::Planning,
            "voice" => SessionMode::Voice,
            _ => SessionMode::Unknown,
        }
    }
}

fn options_for_mode(mode: &SessionMode) -> OllamaGenerateOptions {
    match mode {
        SessionMode::Code => OllamaGenerateOptions { num_ctx: Some(8192), temperature: Some(0.05), top_p: Some(0.9), num_gpu: Some(1) },
        SessionMode::Planning => OllamaGenerateOptions { num_ctx: Some(4096), temperature: Some(0.2), top_p: Some(0.95), num_gpu: Some(1) },
        SessionMode::Voice => OllamaGenerateOptions { num_ctx: Some(2048), temperature: Some(0.6), top_p: Some(0.98), num_gpu: Some(1) },
        SessionMode::Unknown => OllamaGenerateOptions { num_ctx: Some(2048), temperature: Some(0.2), top_p: Some(0.95), num_gpu: Some(1) },
    }
}

async fn stream_ollama_chat(
    ollama_url: &str,
    model: &str,
    system: &str,
    retrieval_block: Option<String>,
    history: &[ChatMsg],
) -> anyhow::Result<BoxedTextStream> {
    // Build a flattened prompt string from history for /api/generate
    fn build_prompt(history: &[ChatMsg], system: &str, retrieval: Option<String>) -> String {
        let mut prompt = String::new();
        if !system.is_empty() {
            prompt.push_str(&format!("System: {}\n", system));
        }
        if let Some(r) = retrieval {
            prompt.push_str(&format!("Relevant memory:\n{}\n", r));
        }
        for m in history {
            match m.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n", m.content)),
                "user" => prompt.push_str(&format!("User: {}\n", m.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n", m.content)),
                _ => prompt.push_str(&format!("{}: {}\n", m.role, m.content)),
            }
        }
        prompt.push_str("Assistant: ");
        prompt
    }

    let prompt = build_prompt(history, system, retrieval_block);

    let req = OllamaGenerateReq {
        model: model.into(),
        prompt,
        stream: true,
        options: Some(OllamaGenerateOptions { num_ctx: Some(2048), temperature: Some(0.2), top_p: Some(0.95), num_gpu: Some(1) }),
    };

    let client = Client::new();
    let res = client
        .post(format!("{}/api/generate", ollama_url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await?
        .error_for_status()?;

    let byte_stream = res.bytes_stream();

    // parse ndjson stream into a Stream of String deltas
    // parse ndjson stream into a Stream of String deltas (expects {"response":"...","done":bool})
    let s = futures::stream::try_unfold(
        (byte_stream, bytes::BytesMut::new()),
        |(mut bs, mut buf)| async move {
            use futures::StreamExt;

            while let Some(chunk) = bs.next().await {
                let chunk = chunk?;
                buf.extend_from_slice(&chunk);

                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line = buf.split_to(pos + 1);
                    let line = &line[..line.len().saturating_sub(1)];
                    if line.is_empty() {
                        continue;
                    }
                    let parsed: OllamaChatStreamChunk = serde_json::from_slice(line)?;
                    if let Some(resp) = parsed.response {
                        if !resp.is_empty() {
                            return Ok(Some((resp, (bs, buf))));
                        }
                    }
                    if parsed.done {
                        return Ok(None);
                    }
                }
            }
            Ok(None)
        },
    );

    Ok(Box::pin(s))
}

/// Variant of the streaming call that accepts explicit generation options.
async fn stream_ollama_chat_with_options(
    ollama_url: &str,
    model: &str,
    system: &str,
    retrieval_block: Option<String>,
    history: &[ChatMsg],
    options: OllamaGenerateOptions,
) -> anyhow::Result<BoxedTextStream> {
    // Build a flattened prompt string from history for /api/generate
    fn build_prompt(history: &[ChatMsg], system: &str, retrieval: Option<String>) -> String {
        let mut prompt = String::new();
        if !system.is_empty() {
            prompt.push_str(&format!("System: {}\n", system));
        }
        if let Some(r) = retrieval {
            prompt.push_str(&format!("Relevant memory:\n{}\n", r));
        }
        for m in history {
            match m.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n", m.content)),
                "user" => prompt.push_str(&format!("User: {}\n", m.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n", m.content)),
                _ => prompt.push_str(&format!("{}: {}\n", m.role, m.content)),
            }
        }
        prompt.push_str("Assistant: ");
        prompt
    }

    let prompt = build_prompt(history, system, retrieval_block);

    let req = OllamaGenerateReq {
        model: model.into(),
        prompt,
        stream: true,
        options: Some(options),
    };

    let client = Client::new();
    let res = client
        .post(format!("{}/api/generate", ollama_url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await?
        .error_for_status()?;

    let byte_stream = res.bytes_stream();

    let s = futures::stream::try_unfold(
        (byte_stream, bytes::BytesMut::new()),
        |(mut bs, mut buf)| async move {
            use futures::StreamExt;

            while let Some(chunk) = bs.next().await {
                let chunk = chunk?;
                buf.extend_from_slice(&chunk);

                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line = buf.split_to(pos + 1);
                    let line = &line[..line.len().saturating_sub(1)];
                    if line.is_empty() {
                        continue;
                    }
                    let parsed: OllamaChatStreamChunk = serde_json::from_slice(line)?;
                    if let Some(resp) = parsed.response {
                        if !resp.is_empty() {
                            return Ok(Some((resp, (bs, buf))));
                        }
                    }
                    if parsed.done {
                        return Ok(None);
                    }
                }
            }
            Ok(None)
        },
    );

    Ok(Box::pin(s))
}

/// Persistence-enabled `/api/chat` handler. Requires `persistence` feature.
#[cfg(feature = "persistence")]
#[axum::debug_handler]
 pub async fn chat_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Extension(ai_bcast): Extension<broadcast::Sender<String>>,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
    Json(req): Json<ChatRequest>,
) -> impl axum::response::IntoResponse {
    let ts = chrono::Utc::now().timestamp_millis();

    // 1. persist user message
    if let Err(e) = store.append_chat_msg(&req.session_id, "user", &req.text) {
        warn!(%e, "failed to append user message");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("append error: {}", e));
    }

    // 1.5 Appeal pre-check: load appeal state for this session. If the session is AwaitingUser, return the stored verdict/state.
    match crate::appeal::load_appeal_state(&store, &req.session_id) {
        Ok(st) => {
            match st {
                crate::council_verdict::AppealState::AwaitingUser { ref verdict_id, .. } => {
                    // try to load the last verdict; if present, return it with 409 Conflict
                    match crate::appeal::load_verdict(&store, &req.session_id, &verdict_id.0) {
                        Ok(v) => {
                            let resp = serde_json::json!({"state": &st, "last_verdict": v});
                            return (StatusCode::CONFLICT, resp.to_string());
                        }
                        Err(_) => {
                            let resp = serde_json::json!({"state": st});
                            return (StatusCode::CONFLICT, resp.to_string());
                        }
                    }
                }
                crate::council_verdict::AppealState::Authorized { .. } => {
                    // authorized: ensure token still valid
                    let now_ms = chrono::Utc::now().timestamp_millis() as u128;
                    if let Err(e) = crate::appeal::token_valid(&st, now_ms) {
                        warn!(%e, "appeal token invalid");
                        // treat as idle (continue) — token validation failure will require resubmission
                    }
                }
                _ => {}
            }
        }
        Err(e) => {
            warn!(%e, "failed to load appeal state");
        }
    }

    // 2. load recent context
    let recent = match store.load_recent_chat(&req.session_id, 20) {
        Ok(v) => v,
        Err(e) => {
            warn!(%e, "failed to load recent chat");
            Vec::new()
        }
    };

    let mut history: Vec<ChatMsg> = recent
        .iter()
        .map(|le| ChatMsg { role: le.speaker.clone(), content: le.content.clone(), ts: le.timestamp_ms })
        .collect();

    // 3. start Ollama stream
    // Prepare Ollama call parameters (used by sentinel speak if needed)
    let ollama_url = req.ollama_url.as_deref().unwrap_or("http://127.0.0.1:11434");
    let model = req.model.as_deref().unwrap_or("deepseek-coder:6.7b");
    let system = req.system.as_deref().unwrap_or("AURA persona");
    let retrieval = req.retrieval.clone();

    // Run Sentinel evaluator synchronously (pure heuristics) to decide whether Sentinel must speak.
    match sentinel_evaluate(&req.text) {
        SentinelDecision::Allow => {
            // continue as normal
        }
        SentinelDecision::AllowWithWarning(reason) => {
            // produce a short sentinel block, persist and broadcast, then continue
            let block = format_sentinel_block(&reason, "ALLOW_WITH_WARNING");
            if let Err(e) = store.append_chat_msg(&req.session_id, "sentinel", &block) {
                warn!(%e, "failed to persist sentinel warning");
            }
            let _ = ai_bcast.send(block.clone());
            // also persist and broadcast to council channel as a SentinelNotice
            if let Ok(payload) = serde_json::to_string(&CouncilWsMsg { kind: "sentinel_notice".to_string(), session_id: req.session_id.clone(), payload: serde_json::json!({"reason": reason, "level": "warning"}) }) {
                let _ = store.append_chat_msg(&req.session_id, "council", &payload);
                let _ = council_bcast.send(payload);
            }
            // continue to main generation
        }
        SentinelDecision::RequireConsent(reason) => {
            // create sentinel speech and return it to the caller, do not proceed with generation
            let system = "You are the Sentinel archetype. Short, exact, non-soothing. Identify risks and demand explicit user confirmation before proceeding.";
            match sentinel_speak(ollama_url, model, system).await {
                Ok(resp) => {
                    let block = resp;
                    let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
                    let _ = ai_bcast.send(block.clone());
                    // persist + broadcast council notice
                    if let Ok(payload) = serde_json::to_string(&CouncilWsMsg { kind: "sentinel_speech".to_string(), session_id: req.session_id.clone(), payload: serde_json::json!({"speech": block}) }) {
                        let _ = store.append_chat_msg(&req.session_id, "council", &payload);
                        let _ = council_bcast.send(payload);
                    }
                    return (StatusCode::OK, block);
                }
                Err(e) => {
                    warn!(%e, "sentinel speak failed");
                    let block = format_sentinel_block(&reason, "REQUIRE_CONSENT");
                    let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
                    let _ = ai_bcast.send(block.clone());
                    if let Ok(payload) = serde_json::to_string(&CouncilWsMsg { kind: "sentinel_notice".to_string(), session_id: req.session_id.clone(), payload: serde_json::json!({"reason": reason, "level": "require_consent"}) }) {
                        let _ = store.append_chat_msg(&req.session_id, "council", &payload);
                        let _ = council_bcast.send(payload);
                    }
                    return (StatusCode::OK, block);
                }
            }
        }
        SentinelDecision::Deny(reason) => {
            let block = format_sentinel_block(&reason, "DENY");
            let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
            let _ = ai_bcast.send(block.clone());
            if let Ok(payload) = serde_json::to_string(&CouncilWsMsg { kind: "verdict".to_string(), session_id: req.session_id.clone(), payload: serde_json::json!({"final_state": "deny", "reason": reason}) }) {
                let _ = store.append_chat_msg(&req.session_id, "council", &payload);
                let _ = council_bcast.send(payload);
            }
            return (StatusCode::FORBIDDEN, block);
        }
    }

    let mut stream = match stream_ollama_chat(ollama_url, model, system, retrieval, &history).await {
        Ok(s) => s,
        Err(e) => {
            warn!(%e, "failed to start ollama stream");
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("stream error: {}", e));
        }
    };

    // 4. stream deltas, broadcast, accumulate
    let mut assistant_buf = String::new();
    while let Some(delta_res) = stream.next().await {
        match delta_res {
            Ok(token) => {
                assistant_buf.push_str(&token);
                let _ = ai_bcast.send(token);
            }
            Err(e) => {
                warn!(%e, "error reading stream chunk");
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("stream chunk error: {}", e));
            }
        }
    }

    // 5. persist assistant message
    if let Err(e) = store.append_chat_msg(&req.session_id, "assistant", &assistant_buf) {
        warn!(%e, "failed to append assistant message");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("append error: {}", e));
    }

    (StatusCode::OK, String::new())
}
