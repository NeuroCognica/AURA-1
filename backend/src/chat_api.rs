use crate::sentinel::{format_sentinel_block, sentinel_evaluate, sentinel_speak, SentinelDecision};
use crate::storage::RocksStore;
use axum::{extract::Extension, extract::Json};
use axum::http::StatusCode;
use futures_util::StreamExt;
use chrono;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};
use crate::intent_stratification::IntentClassifier;

/// Ingress handler: accept chat POST, run Sentinel, and on ALLOW spawn
/// a background task that drives `Ollama` streaming and broadcasts tokens.
#[cfg(feature = "persistence")]
#[axum::debug_handler]
pub async fn chat_handler(
    Extension(store): Extension<Arc<RocksStore>>,
    Extension(ai_bcast): Extension<broadcast::Sender<String>>,
    Extension(council_bcast): Extension<broadcast::Sender<String>>,
    Extension(council_bcast_typed): Extension<
        broadcast::Sender<crate::council_verdict::CouncilEnvelope>,
    >,
    Extension(gen_mgr): Extension<Arc<crate::generation_manager::GenerationManager>>,
    Json(req): Json<crate::ollama::ChatRequest>,
) -> impl axum::response::IntoResponse {
    // persist user message
    if let Err(e) = store.append_chat_msg(&req.session_id, "user", &req.text) {
        warn!(%e, "failed to append user message");
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("append error: {}", e));
    }

    // Appeal pre-check (reuse existing logic)
    match crate::appeal::load_appeal_state(&*store, &req.session_id) {
        Ok(st) => match st {
            crate::council_verdict::AppealState::AwaitingUser { ref verdict_id, .. } => {
                        match crate::appeal::load_verdict(&*store, &req.session_id, &verdict_id.0) {
                    Ok(v) => {
                        let resp = serde_json::json!({"state": &st, "last_verdict": v});
                        return (StatusCode::CONFLICT, resp.to_string());
                    }
                    Err(_) => {
                        let resp = serde_json::json!({"state": st});
                        return (StatusCode::CONFLICT, resp.to_string());
                    }
                }
            }
            crate::council_verdict::AppealState::Authorized { .. } => {
                let now_ms = chrono::Utc::now().timestamp_millis() as u128;
                if let Err(e) = crate::appeal::token_valid(&st, now_ms) {
                    warn!(%e, "appeal token invalid");
                }
            }
            _ => {}
        },
        Err(e) => {
            warn!(%e, "failed to load appeal state");
        }
    }

    // Intent Stratification: Pre-constitutional cognition layer
    // This filters queries before Sentinel to reduce load
    let intent_classifier = IntentClassifier::new();
    let classification = intent_classifier.classify(&req.text);
    
    info!(
        "Intent classification: speculation={:?}, reflection={:?}, binding={:?}, operation={:?}, sensitivity={:?}, confidence={:.2}, requires_sentinel={}",
        classification.axes.speculation_axis,
        classification.axes.reflection_axis,
        classification.axes.binding_axis,
        classification.axes.operation_axis,
        classification.axes.sensitivity_axis,
        classification.confidence,
        classification.requires_sentinel
    );

    // Log routing decision to ledger
    let routing_decision = json!({
        "timestamp_ms": chrono::Utc::now().timestamp_millis(),
        "session_id": &req.session_id,
        "user_input_length": req.text.len(),
        "classification": {
            "speculation": format!("{:?}", classification.axes.speculation_axis),
            "reflection": format!("{:?}", classification.axes.reflection_axis),
            "binding": format!("{:?}", classification.axes.binding_axis),
            "operation": format!("{:?}", classification.axes.operation_axis),
            "sensitivity": format!("{:?}", classification.axes.sensitivity_axis),
            "confidence": classification.confidence,
        },
        "requires_sentinel": classification.requires_sentinel,
    });
    
    if let Err(e) = store.append_chat_msg(&req.session_id, "intent_routing", &routing_decision.to_string()) {
        warn!(%e, "failed to log routing decision");
    }

    // Route based on classification
    if !classification.requires_sentinel {
        // Direct LLM path: Skip Sentinel, proceed directly to generation
        info!("Routing to direct LLM (Sentinel bypassed)");
        
        // Log bypass event
        crate::broadcast::broadcast_council(
            &store,
            &council_bcast,
            Some(&council_bcast_typed),
            &req.session_id,
            "intent_bypass",
            json!({
                "reason": "speculative_or_informational",
                "confidence": classification.confidence,
                "axes": routing_decision["classification"].clone()
            }),
            Some(&gen_mgr),
        );
        
        // Proceed directly to generation (skip sentinel evaluation)
        // (Generation spawning code will follow below)
    } else {
        // Sentinel path: Constitutional review required
        info!("Routing to Sentinel (constitutional review required)");
        // Continue to existing Sentinel evaluation below
    }

    // Sentinel evaluation (only runs if requires_sentinel is true)
    if classification.requires_sentinel {
    match sentinel_evaluate(&req.text) {
        SentinelDecision::Allow => {
            // continue
        }
        SentinelDecision::AllowWithWarning(reason) => {
            let block = format_sentinel_block(&reason, "ALLOW_WITH_WARNING");
            let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
            let _ = ai_bcast.send(block.clone());
            crate::broadcast::broadcast_council(
                &store,
                &council_bcast,
                Some(&council_bcast_typed),
                &req.session_id,
                "sentinel_notice",
                serde_json::json!({"reason": reason, "level": "warning"}),
                Some(&gen_mgr),
            );
        }
        SentinelDecision::RequireConsent(reason) => {
            let ollama_url = req.ollama_url.as_deref().unwrap_or("http://127.0.0.1:11434");
            let model = req.model.as_deref().unwrap_or("deepseek-coder:6.7b");
            let system = "You are the Sentinel archetype. Short, exact, non-soothing. Identify risks and demand explicit user confirmation before proceeding.";
            match sentinel_speak(ollama_url, model, system).await {
                Ok(resp) => {
                    let block = resp;
                    let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
                    let _ = ai_bcast.send(block.clone());
                    crate::broadcast::broadcast_council(
                        &store,
                        &council_bcast,
                        Some(&council_bcast_typed),
                        &req.session_id,
                        "sentinel_speech",
                        serde_json::json!({"speech": block}),
                        Some(&gen_mgr),
                    );
                    return (StatusCode::OK, block);
                }
                Err(e) => {
                    warn!(%e, "sentinel speak failed");
                    let block = format_sentinel_block(&reason, "REQUIRE_CONSENT");
                    let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
                    let _ = ai_bcast.send(block.clone());
                    crate::broadcast::broadcast_council(
                        &store,
                        &council_bcast,
                        Some(&council_bcast_typed),
                        &req.session_id,
                        "sentinel_notice",
                        serde_json::json!({"reason": reason, "level": "require_consent"}),
                        Some(&gen_mgr),
                    );
                    return (StatusCode::OK, block);
                }
            }
        }
        SentinelDecision::Deny(reason) => {
            let block = format_sentinel_block(&reason, "DENY");
            let _ = store.append_chat_msg(&req.session_id, "sentinel", &block);
            let _ = ai_bcast.send(block.clone());
            crate::broadcast::broadcast_council(
                &store,
                &council_bcast,
                Some(&council_bcast_typed),
                &req.session_id,
                "verdict",
                serde_json::json!({"final_state": "deny", "reason": reason}),
                Some(&gen_mgr),
            );
            return (StatusCode::FORBIDDEN, block);
        }
    }
    } // Close the `if classification.requires_sentinel` block

    // Spawn generation via GenerationManager so the manager owns lifecycle.
    let store_b = store.clone();
    let ai_bcast_b = ai_bcast.clone();
    let council_bcast_b = council_bcast.clone();
    let council_bcast_typed_b = council_bcast_typed.clone();
    let gen_mgr_b = gen_mgr.clone();
    // Clone the session id and other request-owned fields so the spawned task
    // does not borrow `req` (avoids E0505 borrowing/move errors).
    let session_id = req.session_id.clone();
    let ollama_url = req.ollama_url.clone().unwrap_or_else(|| "http://127.0.0.1:11434".to_string());
    let model = req.model.clone().unwrap_or_else(|| "deepseek-coder:6.7b".to_string());
    let system = req.system.clone().unwrap_or_else(|| "AURA persona".to_string());
    let retrieval = req.retrieval.clone();

    let sid_for_spawn = session_id.clone();
    let gen_id = gen_mgr
        .spawn_generation(&sid_for_spawn, move |gen_id, cancel| async move {
            let session_id = session_id.clone();
            let ollama_url = ollama_url.clone();
            let model = model.clone();
            let system = system.clone();
            let retrieval = retrieval.clone();
            let store_b = store_b.clone();
            let ai_bcast_b = ai_bcast_b.clone();
            let council_bcast_b = council_bcast_b.clone();
            let council_bcast_typed_b = council_bcast_typed_b.clone();
                // print/log for visibility
                tracing::info!(session_id = %session_id, gen_id = %gen_id, "generation_task_running");
                println!("GENERATION_TASK_RUNNING session_id={} gen_id={}", session_id, gen_id);

                // load recent chat context
                let recent = match store_b.load_recent_chat(&session_id, 20) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(%e, "failed to load recent chat");
                        Vec::new()
                    }
                };

                let history: Vec<crate::ollama::ChatMsg> = recent
                    .iter()
                    .map(|le| crate::ollama::ChatMsg { role: le.speaker.clone(), content: le.content.clone(), ts: le.timestamp_ms })
                    .collect();

                let mut stream = match crate::ollama::stream_ollama_chat(&ollama_url, &model, &system, retrieval, &history).await {
                    Ok(s) => s,
                    Err(e) => {
                            warn!(%e, "failed to start ollama stream in generation task");
                            // For developer/demo flows, emit a short fake token stream so frontends
                            // can be wired and tested even when Ollama is unreachable.
                            let demo_tokens = vec!["Hello", ", world", "! This is a demo."]; 
                            for t in demo_tokens.iter() {
                                let env = json!({"type": "token", "gen_id": gen_id, "payload": t}).to_string();
                                let _ = ai_bcast_b.send(env);
                                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                            }
                            let env_end = json!({"type": "end", "gen_id": gen_id, "reason": format!("stream_error: {}", e)}).to_string();
                            let _ = ai_bcast_b.send(env_end);
                            return;
                        }
                };

            // helpers
            let send_token = |ai_bcast: &broadcast::Sender<String>, gen_id: &str, token: &str| {
                let env = json!({"type": "token", "gen_id": gen_id, "payload": token});
                let _ = ai_bcast.send(env.to_string());
            };
            let send_end = |ai_bcast: &broadcast::Sender<String>, gen_id: &str, reason: &str| {
                let env = json!({"type": "end", "gen_id": gen_id, "reason": reason});
                let _ = ai_bcast.send(env.to_string());
            };

            let mut assistant_buf = String::new();

            loop {
                tokio::select! {
                    biased;
                    _ = cancel.cancelled() => {
                        send_end(&ai_bcast_b, &gen_id, "canceled_by_authority");
                        return;
                    }
                    next = stream.next() => {
                        match next {
                            Some(Ok(token)) => {
                                assistant_buf.push_str(&token);
                                send_token(&ai_bcast_b, &gen_id, &token);
                            }
                            Some(Err(e)) => {
                                warn!(%e, "error reading stream chunk in generation task");
                                send_end(&ai_bcast_b, &gen_id, &format!("stream_chunk_error: {}", e));
                                return;
                            }
                            None => break,
                        }
                    }
                }
            }

            // persist assistant message
            if let Err(e) = store_b.append_chat_msg(&session_id, "assistant", &assistant_buf) {
                warn!(%e, "failed to append assistant message from generation task");
                send_end(&ai_bcast_b, &gen_id, "persist_error");
                return;
            }

            send_end(&ai_bcast_b, &gen_id, "completed");
        })
        .await;

    // Immediately return Accepted with gen_id so client can correlate
    (StatusCode::ACCEPTED, json!({"ok": true, "gen_id": gen_id}).to_string())
}
