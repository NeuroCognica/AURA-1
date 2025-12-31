#![cfg(feature = "persistence")]

use aura_backend::{broadcast, council_verdict, storage};
use axum::{extract::Extension, routing::get, Router};
use axum_server::Server;
use futures_util::{SinkExt, StreamExt};

use std::sync::Arc;
use tempfile::tempdir;
use tokio::net::TcpListener;

#[tokio::test]
async fn ws_replay_end_to_end() -> anyhow::Result<()> {
    use aura_backend::council_verdict::{make_council_envelope, CouncilMsg};
    use aura_backend::storage::RocksStore;
    use tokio_tungstenite::tungstenite::Message as WsMessage;

    // prepare persistence store
    let td = tempdir()?;
    let store = RocksStore::open(td.path())?;
    let store = Arc::new(store);

    // channels
    let (council_bcast_tx, _rx1) = tokio::sync::broadcast::channel::<String>(256);
    let (council_bcast_typed_tx, _rx2) =
        tokio::sync::broadcast::channel::<aura_backend::council_verdict::CouncilEnvelope>(256);

    // Persist N=5 typed envelopes synchronously so replay will see them.
    let sid = "session-test";
    for i in 1..=5u64 {
        let payload = serde_json::json!({"i": i});
        // Build a typed Notice envelope for simplicity
        let env_bytes = serde_json::to_vec(&make_council_envelope(
            sid,
            None,
            i,
            CouncilMsg::Notice(payload.clone()),
        ))?;
        // Use storage helper to persist with seq i (helper will assign seq)
        let seq = store.append_council_envelope_with(sid, |_assigned_seq| env_bytes.clone())?;
        // Also append a legacy chat message for backward compatibility
        let chat = serde_json::json!({"kind": "verdict", "session_id": sid, "payload": payload})
            .to_string();
        let _ = council_bcast_tx.send(chat);
        assert_eq!(seq >= 1, true);
    }

    // Build minimal app exposing ws/council
    let app = Router::new()
        .route("/ws/council", get(broadcast::ws_council_handler))
        .layer(Extension(council_bcast_tx.clone()))
        .layer(Extension(council_bcast_typed_tx.clone()))
        .layer(Extension(Some(store.clone())));

    // bind to ephemeral port (std listener for axum_server)
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = std_listener.local_addr()?;
    let server = Server::from_tcp(std_listener).serve(app.into_make_service());
    tokio::spawn(server);

    // connect websocket
    let url = format!("ws://{}/ws/council", addr);
    let (mut ws_stream, _resp) = tokio_tungstenite::connect_async(&url).await?;

    // send Hello with last_ack = 2 so we expect seq 3..5 replayed
    let hello = serde_json::json!({"type": "hello", "sid": sid, "last_ack": 2}).to_string();
    ws_stream.send(WsMessage::Text(hello)).await?;

    // expect replayed envelopes for seq 3..5 in order
    for expected in 3u64..=5u64 {
        let opt = tokio::time::timeout(std::time::Duration::from_secs(2), ws_stream.next()).await?;
        let some = opt.ok_or_else(|| anyhow::anyhow!("ws closed during replay"))?;
        let msg = some?;
        match msg {
            WsMessage::Text(txt) => {
                let v: serde_json::Value = serde_json::from_str(&txt)?;
                let seq = v.get("seq").and_then(|s| s.as_u64()).expect("seq present");
                assert_eq!(seq, expected);
            }
            other => panic!("unexpected message: {:?}", other),
        }
    }

    // next message must be replay_done
    let opt = tokio::time::timeout(std::time::Duration::from_secs(2), ws_stream.next()).await?;
    let some = opt.ok_or_else(|| anyhow::anyhow!("ws closed before replay_done"))?;
    let msg = some?;
    match msg {
        WsMessage::Text(txt) => {
            let v: serde_json::Value = serde_json::from_str(&txt)?;
            assert_eq!(v.get("type").and_then(|t| t.as_str()), Some("replay_done"));
        }
        other => panic!("expected replay_done, got {:?}", other),
    }

    // Now send a live council message and ensure client receives it after replay
    broadcast::broadcast_council(
        &store,
        &council_bcast_tx,
        Some(&council_bcast_typed_tx),
        sid,
        "verdict",
        serde_json::json!({"i": 6}),
        None,
    );

    // receive seq 6 (skip legacy immediate message if present; typed envelope is sent after persistence)
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut found = false;
    while Instant::now() < deadline {
        let opt = tokio::time::timeout(Duration::from_secs(1), ws_stream.next()).await?;
        let some = match opt {
            Some(s) => s,
            None => continue,
        };
        let msg = some?;
        match msg {
            WsMessage::Text(txt) => {
                let v: serde_json::Value = serde_json::from_str(&txt)?;
                if let Some(seq) = v.get("seq").and_then(|s| s.as_u64()) {
                    assert_eq!(seq, 6);
                    found = true;
                    break;
                } else {
                    // legacy message; keep waiting for typed envelope
                    continue;
                }
            }
            _ => continue,
        }
    }
    if !found {
        panic!("did not receive typed live envelope within timeout");
    }

    Ok(())
}
