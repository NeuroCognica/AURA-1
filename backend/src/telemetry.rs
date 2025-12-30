use serde::{Deserialize, Serialize};
use axum::{extract::ws::{Message, WebSocket, WebSocketUpgrade}, response::IntoResponse, Extension};
use futures_util::StreamExt;
use tracing::{info, warn};
use tokio::sync::watch;
use std::sync::{Arc};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryEnvelope {
    pub r#type: String,
    pub session: String,
    pub ts: f64,
    pub seq: u64,
    pub payload: TelemetryPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TelemetryPayload {
    Pose { pose: Pose },
    Focus { focus: Focus },
    Observe { observe: Observe },
    Empty {},
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Focus {
    pub vec: [f32; 3],
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Observe {
    pub target: String,
    pub duration_ms: u64,
}

pub async fn ws_telemetry_handler(
    ws: WebSocketUpgrade,
    Extension(pose_tx): Extension<watch::Sender<Pose>>,
    Extension(telemetry_counter): Extension<Arc<AtomicU64>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_telemetry_socket(socket, pose_tx, telemetry_counter))
}

async fn handle_telemetry_socket(mut socket: WebSocket, pose_tx: watch::Sender<Pose>, telemetry_counter: Arc<AtomicU64>) {
    info!("telemetry socket connected");

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<TelemetryEnvelope>(&text) {
                    Ok(frame) => {
                        handle_telemetry_frame(frame, &pose_tx, &telemetry_counter).await;
                    }
                    Err(e) => warn!("invalid telemetry frame: {e}"),
                }
            }
            Ok(Message::Binary(_)) => warn!("binary frames not supported (yet)"),
            Ok(Message::Close(_)) => {
                info!("telemetry socket closed");
                break;
            }
            _ => {}
        }
    }
}

async fn handle_telemetry_frame(frame: TelemetryEnvelope, pose_tx: &watch::Sender<Pose>, telemetry_counter: &Arc<AtomicU64>) {
    tracing::debug!(seq = frame.seq, session = %frame.session, "telemetry received");
    telemetry_counter.fetch_add(1, Ordering::Relaxed);

    match frame.payload {
        TelemetryPayload::Pose { pose } => {
            let _ = pose_tx.send(pose);
        }
        TelemetryPayload::Focus { .. } => {
            // future: ingest focus into BCAT seeds
        }
        TelemetryPayload::Observe { .. } => {
            // future: emit observe events into Codex
        }
        TelemetryPayload::Empty {} => {}
    }
}
