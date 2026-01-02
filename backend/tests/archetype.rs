use std::time::Duration;

use serde_json::Value;

use axum::{Router, extract::{Json, Extension}};
use tokio::sync::broadcast;
use std::sync::Arc;
use axum::http::StatusCode;

// Test the activation handler via direct handler invocation (no TCP server needed).
#[tokio::test]
async fn activation_api_oneshot() -> anyhow::Result<()> {
    // Prepare minimal archetypes map with cssVars and audio
    let mut map = std::collections::HashMap::new();
    let sentinel = serde_json::json!({
        "id": "sentinel",
        "cssVars": {"--bg-primary": "#000000", "--accent": "#ff0000"},
        "audio": {"tone": "F#", "duration_ms": 400, "wave": "sine"}
    });
    map.insert("sentinel".to_string(), sentinel);
    let arche = Arc::new(map);

    let (council_tx, _council_rx) = broadcast::channel::<String>(256);
    let (typed_tx, _typed_rx) = broadcast::channel::<aura_backend::council_verdict::CouncilEnvelope>(256);

    use aura_backend::archetype_api::activate_archetype_handler;

    // Call the library handler directly (avoids routing/type issues)
    let payload = serde_json::json!({"archetype": "sentinel", "ritual": true});
    let (status, axum_json) = activate_archetype_handler(
        axum::Json(payload.clone()),
        axum::Extension(arche.clone()),
        axum::Extension(council_tx.clone()),
        axum::Extension(typed_tx.clone()),
    ).await;

    assert!(status.is_success());
    let j = axum_json.0;
    assert_eq!(j.get("archetype").and_then(|v| v.as_str()).unwrap_or(""), "sentinel");
    assert!(j.get("theme_vars").is_some());
    assert!(j.get("audio_signature").is_some());
    assert!(j.get("transition").is_some());

    Ok(())
}

// Test that a websocket client receives the archetype_changed broadcast.
#[tokio::test]
async fn archetype_ws_broadcast() -> anyhow::Result<()> {
    // Prepare map and channels
    let mut map = std::collections::HashMap::new();
    let sentinel = serde_json::json!({
        "id": "sentinel",
        "cssVars": {"--bg-primary": "#000000", "--accent": "#ff0000"},
        "audio": {"tone": "F#", "duration_ms": 400, "wave": "sine"}
    });
    map.insert("sentinel".to_string(), sentinel);
    let arche = Arc::new(map);

    let (council_tx, _council_rx) = broadcast::channel::<String>(256);
    let (typed_tx, _typed_rx) = broadcast::channel::<aura_backend::council_verdict::CouncilEnvelope>(256);

    // Subscribe to the council broadcast channel and invoke the handler directly.
    let mut rx = council_tx.subscribe();

    let rocksdb_path = std::path::Path::new("data/rocksdb");
    let existed_before = rocksdb_path.exists();

    let payload = serde_json::json!({"archetype": "sentinel", "ritual": true});
    let (_status, _json) = aura_backend::archetype_api::activate_archetype_handler(
        axum::Json(payload),
        axum::Extension(arche.clone()),
        axum::Extension(council_tx.clone()),
        axum::Extension(typed_tx.clone()),
    ).await;

    // Await the broadcast message
    let msg = tokio::time::timeout(Duration::from_secs(2), async { rx.recv().await }).await?;
    let s = msg?;
    let env: Value = serde_json::from_str(&s)?;
    let payload = env.get("payload").cloned().unwrap_or(Value::Null);
    if payload.is_object() {
        let archetype = payload.get("archetype").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(archetype, "sentinel");
    } else {
        panic!("unexpected payload shape: {}", payload);
    }

    // Ensure handler did not create RocksDB when persistence is off
    let existed_after = rocksdb_path.exists();
    assert_eq!(existed_after, existed_before, "activate_archetype_handler must not create RocksDB directories when persistence is disabled");

    Ok(())
}
