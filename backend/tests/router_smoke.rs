use axum::{Router, extract::Extension};
use axum::body::Body;
use axum::http::{Request, Method, StatusCode};
use tower::util::ServiceExt;
use tokio::sync::broadcast;
use std::sync::Arc;
use serde_json::Value;

#[tokio::test]
async fn route_smoke() -> anyhow::Result<()> {
    // Minimal archetype map
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

    // Build a router with the same route as main.rs
    let app = Router::new()
        .route(
            "/api/archetype/activate",
            axum::routing::post({
                let arche = arche.clone();
                let council = council_tx.clone();
                let council_typed = typed_tx.clone();
                move |axum::Json(payload): axum::Json<serde_json::Value>| {
                    let arche = arche.clone();
                    let council = council.clone();
                    let council_typed = council_typed.clone();
                    async move {
                        aura_backend::archetype_api::activate_archetype_handler(
                            axum::Json(payload),
                            axum::Extension(arche),
                            axum::Extension(council),
                            axum::Extension(council_typed),
                        )
                        .await
                    }
                }
            }),
        )
        .layer(Extension(arche.clone()))
        .layer(Extension(council_tx.clone()))
        .layer(Extension(typed_tx.clone()));

    let payload = serde_json::json!({"archetype":"sentinel","ritual":true});
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/archetype/activate")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    use axum::body::to_bytes;
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await?;
    let j: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(j.get("archetype").and_then(|v| v.as_str()).unwrap_or(""), "sentinel");

    Ok(())
}
