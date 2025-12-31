use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Extension;
use futures_util::StreamExt;
use std::sync::Arc;

// `storage::RocksStore` is only available when the `persistence` feature
// is enabled. Guard the import so this file compiles regardless of
// whether the feature is active.
#[cfg(feature = "persistence")]
use crate::storage::RocksStore;
use serde_json::Value as JsonValue;
use crate::council_verdict::{CouncilWsMsg, CouncilEnvelope, CouncilClientMsg, CouncilMsgType, now_ms};
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
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    // Upgrade and hand off to handler that attaches sequencing/acking semantics
    ws.on_upgrade(move |socket| handle_ai_socket(socket, ai_bcast, council_bcast))
}

#[cfg(not(feature = "persistence"))]
pub async fn ws_council_handler(
    ws: WebSocketUpgrade,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_council_socket(socket, council_bcast))
}

#[cfg(feature = "persistence")]
pub async fn ws_council_handler(
    ws: WebSocketUpgrade,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
    Extension(council_bcast_typed): Extension<broadcast::Sender<crate::council_verdict::CouncilEnvelope>>,
    Extension(store): Extension<Option<Arc<RocksStore>>>,
) -> impl IntoResponse {
    let store = store.expect("persistence store created");
    ws.on_upgrade(move |socket| handle_council_socket_typed(socket, council_bcast_typed.subscribe(), council_bcast.subscribe(), store.clone()))
}

/// Build a minimal, non-authoritative AI notice from a stored `CouncilWsMsg` JSON string.
/// Returns `Some(String)` JSON notice when `msg` is an `interrupt`, otherwise `None`.
pub fn build_ai_interrupt_notice_from_council(msg: &str) -> Option<String> {
    if let Ok(cmsg) = serde_json::from_str::<crate::council_verdict::CouncilWsMsg>(msg) {
        if cmsg.kind != "interrupt" {
            return None;
        }
        // Extract minimal fields: kind, scope, reason, correlation (if present)
        let payload = cmsg.payload;
        let kind = payload.get("kind").and_then(|v| v.as_str()).unwrap_or("interrupt");
        let scope = payload.get("scope").cloned().unwrap_or(serde_json::json!(null));
        let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let correlation = payload.get("correlation").cloned().unwrap_or(serde_json::json!(null));

        let notice = serde_json::json!({
            "type": "notice",
            "notice": "interrupt",
            "payload": { "kind": kind, "scope": scope, "reason": reason, "correlation": correlation }
        });
        return Some(notice.to_string());
    }
    None
}

async fn handle_council_socket(mut socket: WebSocket, council_bcast: broadcast::Sender<String>) {
    info!("council client socket connected");
    let mut rx = council_bcast.subscribe();
    // sequencing for council events (per-connection view)
    let mut seq: u64 = 0;
    let mut last_acked: u64 = 0;
    let mut last_sent: u64 = 0;

    loop {
        tokio::select! {
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(msg) => {
                        // msg is the stored CouncilWsMsg JSON string produced by broadcast_council.
                        // Try to parse it and build a richer envelope for clients.
                        if let Ok(cmsg) = serde_json::from_str::<CouncilWsMsg>(&msg) {
                            seq = seq.wrapping_add(1);
                            last_sent = seq;
                            // map kind string to CouncilMsgType (best-effort)
                            let mtype = match cmsg.kind.as_str() {
                                "verdict" => CouncilMsgType::Verdict,
                                "appeal_state" => CouncilMsgType::AppealState,
                                "sentinel_notice" => CouncilMsgType::SentinelNotice,
                                "interrupt" => CouncilMsgType::Interrupt,
                                _ => CouncilMsgType::SentinelNotice,
                            };

                            let env = CouncilEnvelope {
                                seq,
                                msg_type: mtype,
                                ts_ms: now_ms(),
                                sid: cmsg.session_id.clone(),
                                vid: None,
                                payload: cmsg.payload,
                            };

                            if let Ok(s) = serde_json::to_string(&env) {
                                if socket.send(Message::Text(s)).await.is_err() {
                                    break;
                                }
                            }

                            // simple lag detection
                            let lag = last_sent.saturating_sub(last_acked);
                            if lag > 2000 {
                                let _ = socket.send(Message::Close(None)).await;
                                break;
                            }
                        } else {
                            // fallback: send raw wrapped council message
                            seq = seq.wrapping_add(1);
                            last_sent = seq;
                            let envelope = serde_json::json!({"seq": seq, "type": "council", "ts_ms": now_ms(), "payload": msg});
                            let s = envelope.to_string();
                            if socket.send(Message::Text(s)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("council subscriber lagged by {} messages", n);
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            ws_msg = socket.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(client_msg) = serde_json::from_str::<CouncilClientMsg>(&txt) {
                            match client_msg {
                                CouncilClientMsg::Ack { ack, sid: _ } => {
                                    if ack > last_acked { last_acked = ack; }
                                }
                                CouncilClientMsg::Hello { sid: _, last_ack } => {
                                    if let Some(a) = last_ack { last_acked = a; }
                                }
                            }
                        } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if v.get("ping").is_some() {
                                let _ = socket.send(Message::Text(serde_json::json!({"pong": true, "last_acked": last_acked}).to_string())).await;
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

#[cfg(feature = "persistence")]
async fn handle_council_socket_typed(
    mut socket: WebSocket,
    mut typed_rx: broadcast::Receiver<crate::council_verdict::CouncilEnvelope>,
    mut legacy_rx: broadcast::Receiver<String>,
    store: Arc<RocksStore>,
) {
    use std::time::Duration;
    use crate::council_verdict::CouncilClientMsg;
    info!("council client socket connected (typed)");

    // Handshake: wait for Hello message with `sid` and optional `last_ack`.
    // Timeout so clients that don't send Hello still receive live stream.
    let mut sid: Option<String> = None;
    let mut last_acked: u64 = 0;

    if let Ok(Some(Ok(Message::Text(txt)))) = tokio::time::timeout(Duration::from_secs(5), socket.next()).await {
        if let Ok(client_msg) = serde_json::from_str::<CouncilClientMsg>(&txt) {
            if let CouncilClientMsg::Hello { sid: s, last_ack } = client_msg {
                sid = Some(s);
                if let Some(a) = last_ack { last_acked = a; }
            }
        }
    }

    // If we have a session id, attempt replay from store.
    if let Some(sid_val) = sid.clone() {
        // read last seq for session
        let last_key = format!("sess_council_last:{}", sid_val);
        if let Ok(Some(b)) = store.get_bytes(last_key.as_bytes()) {
            if b.len() == 8 {
                let last_seq = u64::from_be_bytes(b.as_slice().try_into().unwrap());
                if last_seq > last_acked {
                    let max_replay: u64 = 2000;
                    let needed = last_seq.saturating_sub(last_acked);
                    if needed > max_replay {
                        let notice = serde_json::json!({"type": "replay_too_large", "sid": sid_val, "from": last_acked + 1, "to": last_seq});
                        let _ = socket.send(Message::Text(notice.to_string())).await;
                        let _ = socket.send(Message::Close(None)).await;
                        return;
                    }

                    // load persisted envelopes and stream them in order
                    if let Ok(items) = store.load_council_range(&sid_val, last_acked + 1, last_seq) {
                        for b in items {
                            if let Ok(s) = String::from_utf8(b) {
                                if socket.send(Message::Text(s)).await.is_err() {
                                    return;
                                }
                            }
                        }
                        // Replay done marker
                        let marker = serde_json::json!({"type": "replay_done", "sid": sid_val, "seq": last_seq});
                        let _ = socket.send(Message::Text(marker.to_string())).await;
                    }
                }
            }
        }
    }

    // After replay, forward live typed envelopes to client and also listen for client acks/pings.
    loop {
        tokio::select! {
            biased;
            // prefer typed envelopes
            recv = typed_rx.recv() => {
                match recv {
                    Ok(env) => {
                        if socket.send(Message::Text(serde_json::to_string(&env).unwrap_or_default())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("typed council subscriber lagged by {} messages", n);
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // fallback: legacy string messages (should be rare once typed migration completes)
            recv2 = legacy_rx.recv() => {
                match recv2 {
                    Ok(s) => {
                        if socket.send(Message::Text(s)).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("legacy council subscriber lagged by {} messages", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {}
                }
            }
            ws_msg = socket.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(client_msg) = serde_json::from_str::<serde_json::Value>(&txt) {
                            if client_msg.get("ping").is_some() {
                                let _ = socket.send(Message::Text(serde_json::json!({"pong": true, "last_acked": last_acked}).to_string())).await;
                                continue;
                            }
                        }

                        if let Ok(client_msg) = serde_json::from_str::<CouncilClientMsg>(&txt) {
                            match client_msg {
                                CouncilClientMsg::Ack { ack, sid: _ } => {
                                    if ack > last_acked { last_acked = ack; }
                                }
                                CouncilClientMsg::Hello { sid: _, last_ack } => {
                                    if let Some(a) = last_ack { last_acked = a; }
                                }
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
    info!("council client socket disconnected (typed)");
}

async fn handle_ai_socket(mut socket: WebSocket, ai_bcast: broadcast::Sender<String>, council_bcast: broadcast::Sender<String>) {
    info!("ai client socket connected");
    let mut rx = ai_bcast.subscribe();
    let mut council_rx = council_bcast.subscribe();
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
            // Listen for council events and forward minimal interrupt notices to ai client
            council = council_rx.recv() => {
                match council {
                    Ok(msg) => {
                        // try parse as CouncilWsMsg; only act on kind == "interrupt"
                        if let Ok(cmsg) = serde_json::from_str::<crate::council_verdict::CouncilWsMsg>(&msg) {
                                if cmsg.kind == "interrupt" {
                                    // Build a minimal, non-authoritative notice for ai clients.
                                    let minimal = crate::broadcast::build_ai_interrupt_notice_from_council(&msg);
                                    if let Some(n) = minimal {
                                        let _ = socket.send(Message::Text(n)).await;
                                    }
                                }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("ai-council subscription lagged by {} messages", n);
                        // don't force close; just warn
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {},
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

/// Helper to persist and broadcast a Council message.
/// Serializes a `CouncilWsMsg`, appends it to the session `council` chat log,
/// and sends the JSON string on the provided `council_bcast` channel.
// Implementation when `persistence` feature is enabled: persist to RocksDB
// and broadcast the stored JSON string.
#[cfg(feature = "persistence")]
pub fn broadcast_council(
    store: &Arc<RocksStore>,
    council_bcast: &broadcast::Sender<String>,
    council_bcast_typed: Option<&broadcast::Sender<crate::council_verdict::CouncilEnvelope>>,
    session_id: &str,
    kind: &str,
    payload: JsonValue,
    gen_mgr: Option<&Arc<crate::generation_manager::GenerationManager>>,
) {
    // Build typed message
    let msg = crate::council_verdict::CouncilWsMsg {
        kind: kind.to_string(),
        session_id: session_id.to_string(),
        payload: payload.clone(),
    };

    match serde_json::to_string(&msg) {
        Ok(s) => {
            // persist the human-readable council chat entry as before
            if let Err(e) = store.append_chat_msg(session_id, "council", &s) {
                warn!(%e, "failed to append council message");
            }

            // Broadcast the existing raw JSON string for backward compatibility
            if let Err(e) = council_bcast.send(s.clone()) {
                warn!("council broadcast send failed: {}", e);
            }

            // If a typed channel is provided, build a typed envelope and persist it
            if let Some(typed) = council_bcast_typed {
                // Try to map known kinds to typed messages; fallback to Notice
                use crate::council_verdict::{CouncilMsg, make_council_envelope};

                let typed_msg = match kind {
                    "verdict" => {
                        if let Ok(v) = serde_json::from_value::<crate::council_verdict::CouncilVerdict>(payload.clone()) {
                            CouncilMsg::Verdict(v)
                        } else {
                            CouncilMsg::Notice(payload.clone())
                        }
                    }
                    "appeal_state" => {
                        if let Ok(sv) = serde_json::from_value::<crate::council_verdict::AppealState>(payload.clone()) {
                            CouncilMsg::AppealState(sv)
                        } else {
                            CouncilMsg::Notice(payload.clone())
                        }
                    }
                    "interrupt" => {
                        if let Ok(ip) = serde_json::from_value::<crate::council_verdict::InterruptPayload>(payload.clone()) {
                            CouncilMsg::Interrupt(ip)
                        } else {
                            CouncilMsg::Notice(payload.clone())
                        }
                    }
                    _ => CouncilMsg::Notice(payload.clone()),
                };

                // Persist typed envelope atomically and broadcast typed envelope.
                // Storage helper will allocate a new per-session seq and atomically
                // write the envelope bytes along with updating session state.
                let store = store.clone();
                let sid = session_id.to_string();
                let typed_sender = typed.clone();
                let payload_clone = payload.clone();
                tokio::spawn(async move {
                    match store.append_council_envelope_with(&sid, |seq| {
                        let env = make_council_envelope(&sid, None, seq, typed_msg.clone());
                        serde_json::to_vec(&env).unwrap_or_else(|_| payload_clone.to_string().into_bytes())
                    }) {
                        Ok(seq) => {
                            // Rebuild the envelope with the assigned seq and send it on the typed channel.
                            let env = make_council_envelope(&sid, None, seq, typed_msg.clone());
                            let _ = typed_sender.send(env);
                        }
                        Err(e) => warn!(%e, "failed to persist typed council envelope"),
                    }
                });
            }

            // Enforcement hook: if this council message requires blocking, cancel any active generation
            if let Some(gm) = gen_mgr {
                // Decide blocking criteria: verdict denies or require_consent, or interrupt
                match kind {
                    "verdict" => {
                        if let Some(decision) = payload.get("final_state").and_then(|v| v.as_str()) {
                            if decision == "deny" || decision == "require_consent" {
                                let sid = session_id.to_string();
                                let gm = gm.clone();
                                tokio::spawn(async move { gm.cancel(&sid).await; });
                            }
                        } else if let Some(decision) = payload.get("decision").and_then(|v| v.as_str()) {
                            if decision == "deny" || decision == "require_consent" {
                                let sid = session_id.to_string();
                                let gm = gm.clone();
                                tokio::spawn(async move { gm.cancel(&sid).await; });
                            }
                        }
                    }
                    "interrupt" => {
                        let sid = session_id.to_string();
                        let gm = gm.clone();
                        tokio::spawn(async move { gm.cancel(&sid).await; });
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            warn!(%e, "failed to serialize council message");
        }
    }
}

// Fallback implementation when `persistence` is NOT enabled. This keeps a
// compatible symbol so callers need not be changed for builds that omit
// persistence. We skip any RocksDB persistence and only broadcast the
// message; enforcement cancellation logic is preserved.
#[cfg(not(feature = "persistence"))]
pub fn broadcast_council(
    _store: &Arc<()>,
    council_bcast: &broadcast::Sender<String>,
    council_bcast_typed: Option<&broadcast::Sender<crate::council_verdict::CouncilEnvelope>>,
    session_id: &str,
    kind: &str,
    payload: JsonValue,
    gen_mgr: Option<&Arc<crate::generation_manager::GenerationManager>>,
) {
    // Build typed message
    let msg = crate::council_verdict::CouncilWsMsg {
        kind: kind.to_string(),
        session_id: session_id.to_string(),
        payload: payload.clone(),
    };

    match serde_json::to_string(&msg) {
        Ok(s) => {
            if let Err(e) = council_bcast.send(s.clone()) {
                warn!("council broadcast send failed: {}", e);
            }

            // Also publish typed envelope when channel provided
            if let Some(typed) = council_bcast_typed {
                let env = crate::council_verdict::CouncilEnvelope {
                    seq: 0,
                    msg_type: crate::council_verdict::CouncilMsgType::SentinelNotice,
                    ts_ms: crate::council_verdict::now_ms(),
                    sid: session_id.to_string(),
                    vid: None,
                    payload: payload.clone(),
                };
                let _ = typed.send(env);
            }

            // Enforcement hook: preserve cancellation behavior even without persistence
            if let Some(gm) = gen_mgr {
                match kind {
                    "verdict" => {
                        if let Some(decision) = payload.get("final_state").and_then(|v| v.as_str()) {
                            if decision == "deny" || decision == "require_consent" {
                                let sid = session_id.to_string();
                                let gm = gm.clone();
                                tokio::spawn(async move { gm.cancel(&sid).await; });
                            }
                        } else if let Some(decision) = payload.get("decision").and_then(|v| v.as_str()) {
                            if decision == "deny" || decision == "require_consent" {
                                let sid = session_id.to_string();
                                let gm = gm.clone();
                                tokio::spawn(async move { gm.cancel(&sid).await; });
                            }
                        }
                    }
                    "interrupt" => {
                        let sid = session_id.to_string();
                        let gm = gm.clone();
                        tokio::spawn(async move { gm.cancel(&sid).await; });
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            warn!(%e, "failed to serialize council message");
        }
    }
}
