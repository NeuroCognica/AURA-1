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

/// Minimal `/api/chat` handler. Expects JSON with: {"ollama_url", "model", "system", "history": [{role,content,ts}], "retrieval": optional string}
pub async fn chat_handler(
    Json(payload): Json<serde_json::Value>,
    Extension(ai_bcast): Extension<broadcast::Sender<String>>,
) -> axum::response::Response {
    let ollama_url = payload
        .get("ollama_url")
        .and_then(|v| v.as_str())
        .unwrap_or("http://127.0.0.1:11434");
    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("llama2");
    let system = payload
        .get("system")
        .and_then(|v| v.as_str())
        .unwrap_or("AURA persona");

    let retrieval = payload.get("retrieval").and_then(|v| v.as_str()).map(|s| s.to_string());

    let history: Vec<ChatMsg> = payload
        .get("history")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|it| {
                    let role = it.get("role").and_then(|r| r.as_str())?.to_string();
                    let content = it.get("content").and_then(|c| c.as_str())?.to_string();
                    let ts = it.get("ts").and_then(|t| t.as_i64()).unwrap_or(0);
                    Some(ChatMsg { role, content, ts })
                })
                .collect()
        })
        .unwrap_or_default();

    // Spawn a task to run the Ollama stream and broadcast deltas.
    let ollama_url = ollama_url.to_string();
    let model = model.to_string();
    let system = system.to_string();
    let history_clone = history.clone();
    let retrieval_clone = retrieval.clone();
    let bcast = ai_bcast.clone();

    tokio::spawn(async move {
        match stream_ollama_chat(&ollama_url, &model, &system, retrieval_clone, &history_clone).await {
            Ok(mut s) => {
                while let Some(chunk) = s.next().await {
                    match chunk {
                        Ok(text) => {
                            debug!(%text, "ollama delta");
                            if let Err(e) = bcast.send(text.clone()) {
                                warn!(%e, "broadcast send failed");
                            }
                        }
                        Err(e) => {
                            warn!(%e, "ollama stream chunk parse error");
                            break;
                        }
                    }
                }
                info!("ollama stream completed");
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
