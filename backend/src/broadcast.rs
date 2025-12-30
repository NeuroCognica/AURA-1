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
