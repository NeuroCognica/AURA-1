use axum::{extract::Json, Extension};
use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub role: String,
    pub content: String,
    pub ts: i64,
}

#[derive(Serialize)]
struct OllamaChatReq {
    model: String,
    messages: Vec<OllamaMsg>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<String>,
}

#[derive(Serialize)]
struct OllamaMsg {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatStreamChunk {
    #[serde(default)]
    message: Option<OllamaAssistantMsg>,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize)]
struct OllamaAssistantMsg {
    content: String,
}

async fn stream_ollama_chat(
    ollama_url: &str,
    model: &str,
    system: &str,
    retrieval_block: Option<String>,
    history: &[ChatMsg],
) -> anyhow::Result<impl futures::Stream<Item = anyhow::Result<String>>> {
    let mut messages: Vec<OllamaMsg> = Vec::new();

    messages.push(OllamaMsg {
        role: "system".into(),
        content: system.into(),
    });

    if let Some(rag) = retrieval_block {
        messages.push(OllamaMsg {
            role: "system".into(),
            content: format!("Relevant memory:\n{}", rag),
        });
    }

    for m in history {
        messages.push(OllamaMsg {
            role: m.role.clone(),
            content: m.content.clone(),
        });
    }

    let req = OllamaChatReq {
        model: model.into(),
        messages,
        stream: true,
        keep_alive: Some("30m".into()),
    };

    let client = Client::new();
    let res = client
        .post(format!("{}/api/chat", ollama_url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await?
        .error_for_status()?;

    let byte_stream = res.bytes_stream();

    // parse ndjson stream into a Stream of String deltas
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
                    if let Some(msg) = parsed.message {
                        if !msg.content.is_empty() {
                            return Ok(Some((msg.content, (bs, buf))));
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

    Ok(s)
}

/// Persistence-enabled `/api/chat` handler. Requires `persistence` feature.
#[cfg(feature = "persistence")]
pub async fn chat_handler(
    Json(payload): Json<serde_json::Value>,
    Extension(ai_bcast): Extension<broadcast::Sender<String>>,
    Extension(store): Extension<std::sync::Arc<crate::RocksStore>>,
) -> axum::response::Response {
    // Require `session_id` and `text` fields
    let session_id = match payload.get("session_id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::boxed(axum::body::Full::from("missing session_id")))
                .unwrap();
        }
    };

    let text = match payload.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::BAD_REQUEST)
                .body(axum::body::boxed(axum::body::Full::from("missing text")))
                .unwrap();
        }
    };

    let ollama_url = payload
        .get("ollama_url")
        .and_then(|v| v.as_str())
        .unwrap_or("http://127.0.0.1:11434")
        .to_string();
    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("llama2")
        .to_string();
    let system = payload
        .get("system")
        .and_then(|v| v.as_str())
        .unwrap_or("AURA persona")
        .to_string();

    let retrieval = payload.get("retrieval").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Append user message to session (append-only)
    if let Err(e) = store.append_chat_msg(&session_id, "user", &text) {
        warn!(%e, "failed to append user message to RocksDB");
    }

    // Load recent history for context (last 20 messages)
    let recent = match store.load_recent_chat(&session_id, 20) {
        Ok(v) => v,
        Err(e) => {
            warn!(%e, "failed to load recent chat from RocksDB");
            Vec::new()
        }
    };

    // Map storage entries to ChatMsg for the Ollama request
    let mut history: Vec<ChatMsg> = recent
        .iter()
        .map(|le| ChatMsg { role: le.speaker.clone(), content: le.content.clone(), ts: le.timestamp_ms })
        .collect();

    // Also include any explicit `history` field if provided (appended after persisted history)
    if let Some(arr) = payload.get("history").and_then(|v| v.as_array()) {
        for it in arr {
            if let (Some(role), Some(content)) = (it.get("role").and_then(|r| r.as_str()), it.get("content").and_then(|c| c.as_str())) {
                history.push(ChatMsg { role: role.to_string(), content: content.to_string(), ts: it.get("ts").and_then(|t| t.as_i64()).unwrap_or(0) });
            }
        }
    }

    // Spawn background task: stream Ollama deltas, broadcast, accumulate, then persist assistant message.
    let bcast = ai_bcast.clone();
    let session_clone = session_id.clone();
    let store_clone = store.clone();
    let retrieval_clone = retrieval.clone();
    let system_clone = system.clone();
    let model_clone = model.clone();
    let ollama_url_clone = ollama_url.clone();

    tokio::spawn(async move {
        let mut assistant_accum = String::new();
        match stream_ollama_chat(&ollama_url_clone, &model_clone, &system_clone, retrieval_clone, &history).await {
            Ok(mut s) => {
                while let Some(chunk) = s.next().await {
                    match chunk {
                        Ok(text) => {
                            // Broadcast delta to clients
                            if let Err(e) = bcast.send(text.clone()) {
                                warn!(%e, "broadcast send failed");
                            }
                            assistant_accum.push_str(&text);
                        }
                        Err(e) => {
                            warn!(%e, "ollama stream chunk parse error");
                            break;
                        }
                    }
                }

                info!("ollama stream completed; persisting assistant message");
                if !assistant_accum.is_empty() {
                    if let Err(e) = store_clone.append_chat_msg(&session_clone, "assistant", &assistant_accum) {
                        warn!(%e, "failed to append assistant message to RocksDB");
                    }
                }
            }
            Err(e) => {
                warn!(%e, "failed to start ollama stream");
            }
        }
    });

    axum::response::Response::builder()
        .status(axum::http::StatusCode::ACCEPTED)
        .body(axum::body::boxed(axum::body::Full::from("streaming")))
        .unwrap()
}
