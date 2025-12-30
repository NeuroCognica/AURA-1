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
    ws.on_upgrade(move |socket| handle_ai_socket(socket, ai_bcast))
}

async fn handle_ai_socket(mut socket: WebSocket, ai_bcast: broadcast::Sender<String>) {
    info!("ai client socket connected");
    let mut rx = ai_bcast.subscribe();

    loop {
        tokio::select! {
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(delta) => {
                        if socket.send(Message::Text(delta)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // client fell behind; skip
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // handle client-side messages (ignore payload, allow disconnect)
            ws_msg = socket.next() => {
                if ws_msg.is_none() {
                    break;
                }
            }
        }
    }
    info!("ai client socket disconnected");
}
