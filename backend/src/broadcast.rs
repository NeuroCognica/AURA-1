use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Extension;
use futures_util::StreamExt;
use futures_util::sink::SinkExt;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub async fn ws_pose_ingest(
    ws: WebSocketUpgrade,
    Extension(pose_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ingest(socket, pose_bcast))
}

pub async fn ws_voice_ingest(
    ws: WebSocketUpgrade,
    Extension(voice_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ingest(socket, voice_bcast))
}

async fn handle_ingest(mut socket: WebSocket, bcast: broadcast::Sender<String>) {
    info!("ingest socket connected");
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // forward raw JSON string to broadcast channel; subscribers decide how to parse
                if let Err(e) = bcast.send(text) {
                    warn!("broadcast send failed: {e}");
                }
            }
            Ok(Message::Binary(_)) => {
                warn!("binary ingest frames not supported");
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }
    info!("ingest socket disconnected");
}

pub async fn ws_pose_client(
    ws: WebSocketUpgrade,
    Extension(pose_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_client(socket, pose_bcast.subscribe()))
}

pub async fn ws_voice_client(
    ws: WebSocketUpgrade,
    Extension(voice_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_client(socket, voice_bcast.subscribe()))
}

async fn handle_client(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    info!("client socket connected");
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("subscriber lagged by {n} messages");
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
    info!("client socket disconnected");
}

// Read-only WS endpoint for AI token deltas. Mirrors pose/voice client behavior.
pub async fn ws_ai_handler(
    ws: WebSocketUpgrade,
    Extension(ai_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    // Upgrade and hand off to handler that attaches sequencing/acking semantics
    ws.on_upgrade(move |socket| handle_ai_socket(socket, ai_bcast))
}

pub async fn ws_council_handler(
    ws: WebSocketUpgrade,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_council_socket(socket, council_bcast))
}

async fn handle_council_socket(mut socket: WebSocket, council_bcast: broadcast::Sender<String>) {
    info!("council client socket connected");
    let mut rx = council_bcast.subscribe();

    // sequencing for council events
    let mut seq: u64 = 0;
    loop {
        tokio::select! {
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(msg) => {
                        seq = seq.wrapping_add(1);
                        let envelope = serde_json::json!({"seq": seq, "type": "council", "payload": msg});
                        let s = envelope.to_string();
                        if socket.send(Message::Text(s)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("council subscriber lagged by {} messages", n);
                        let notice = serde_json::json!({"type": "notice", "notice": "lag", "lag_count": n, "seq": seq}).to_string();
                        let _ = socket.send(Message::Text(notice)).await;
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            ws_msg = socket.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if let Some(ack) = v.get("ack").and_then(|a| a.as_u64()) {
                                info!("council ack {} received", ack);
                            }
                            if v.get("ping").is_some() {
                                let _ = socket.send(Message::Text(serde_json::json!({"pong": true, "seq": seq}).to_string())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
    info!("council client socket disconnected");
}

async fn handle_ai_socket(mut socket: WebSocket, ai_bcast: broadcast::Sender<String>) {
    info!("ai client socket connected");
    let mut rx = ai_bcast.subscribe();
    // Simple sequencing: attach a monotonically increasing seq to outgoing deltas.
    let mut seq: u64 = 0;
    loop {
        tokio::select! {
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(delta) => {
                        seq = seq.wrapping_add(1);
                        let envelope = serde_json::json!({"seq": seq, "type": "delta", "payload": delta});
                        let s = envelope.to_string();
                        if socket.send(Message::Text(s)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("ai subscriber lagged by {} messages", n);
                        // notify client about lag; include current seq so client may reconnect/resync
                        let notice = serde_json::json!({"type": "notice", "notice": "lag", "lag_count": n, "seq": seq}).to_string();
                        let _ = socket.send(Message::Text(notice)).await;
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // handle client-side messages (acks, heartbeats, or disconnect)
            ws_msg = socket.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(txt))) => {
                        // try parse ack JSON {"ack": n}
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if let Some(ack) = v.get("ack").and_then(|a| a.as_u64()) {
                                info!("received ack {} from ai client", ack);
                                // ack processing could be extended to manage per-client replay state
                            }
                            // support simple ping
                            if v.get("ping").is_some() {
                                let _ = socket.send(Message::Text(serde_json::json!({"pong": true, "seq": seq}).to_string())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
    info!("ai client socket disconnected");
}
