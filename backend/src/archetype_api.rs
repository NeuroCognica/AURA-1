use axum::{extract::Extension, Json};
use serde_json::Value as JsonValue;
use tokio::sync::broadcast;
use axum::http::StatusCode;

pub async fn activate_archetype_handler(
    Json(payload): Json<JsonValue>,
    Extension(arche): Extension<std::sync::Arc<std::collections::HashMap<String, JsonValue>>>,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
    Extension(council_bcast_typed): Extension<broadcast::Sender<crate::council_verdict::CouncilEnvelope>>,
) -> (StatusCode, Json<JsonValue>) {
    let archetype_str = payload
        .get("archetype")
        .and_then(|v| v.as_str())
        .unwrap_or("architect");
    let ritual = payload.get("ritual").and_then(|v| v.as_bool()).unwrap_or(false);

    let key = archetype_str.to_string();
    let theme_vars = arche
        .get(&key)
        .and_then(|v| v.get("cssVars"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let audio_sig = arche
        .get(&key)
        .and_then(|v| v.get("audio"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"tone":"F#","duration_ms":400,"wave":"sine"}));

    let transition = if ritual {
        serde_json::json!({"mode": "ritual", "duration_ms": 600})
    } else {
        serde_json::json!({"mode": "instant", "duration_ms": 0})
    };

    let resp = serde_json::json!({
        "archetype": archetype_str,
        "theme_vars": theme_vars,
        "audio_signature": audio_sig,
        "transition": transition
    });

    let msg = crate::council_verdict::CouncilWsMsg {
        kind: "archetype_changed".to_string(),
        session_id: "system".to_string(),
        payload: resp.clone(),
    };

    if let Ok(s) = serde_json::to_string(&msg) {
        let _ = council_bcast.send(s.clone());
    }

    // typed envelope publish (best-effort)
    let env = crate::council_verdict::CouncilEnvelope {
        seq: 0,
        msg_type: crate::council_verdict::CouncilMsgType::SentinelNotice,
        ts_ms: crate::council_verdict::now_ms(),
        sid: "system".to_string(),
        vid: None,
        payload: resp.clone(),
    };
    let _ = council_bcast_typed.send(env);

    (StatusCode::OK, Json(resp))
}
