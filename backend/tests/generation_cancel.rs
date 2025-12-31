use tempfile::TempDir;
use aura_backend::storage::RocksStore;
use tokio::sync::broadcast;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn generation_is_cancelled_on_blocking_council_msg() {
    // Prepare store and channels
    let td = TempDir::new().unwrap();
    let path = td.path().to_path_buf();
    let store = RocksStore::open(path).expect("open store");
    let store = Arc::new(store);

    let (ai_tx, mut ai_rx) = broadcast::channel::<String>(32);
    let (council_tx, _council_rx) = broadcast::channel::<String>(32);

    // Generation manager and start a generation for session "s1"
    let gen_mgr = Arc::new(aura_backend::generation_manager::GenerationManager::new());
    let (gen_id, cancel) = gen_mgr.start_new("s1").await;

    // Spawn a fake generator that emits tokens until cancelled, then emits an end envelope.
    let ai_tx_clone = ai_tx.clone();
    let gen_id_clone = gen_id.clone();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_clone.cancelled() => {
                    let env = serde_json::json!({"type": "end", "gen_id": gen_id_clone, "reason": "canceled_by_authority"});
                    let _ = ai_tx_clone.send(env.to_string());
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    let env = serde_json::json!({"type": "token", "gen_id": gen_id_clone, "payload": "x"});
                    let _ = ai_tx_clone.send(env.to_string());
                }
            }
        }
    });

    // verify we receive at least one token before cancellation
    let first = tokio::time::timeout(Duration::from_secs(1), async { ai_rx.recv().await }).await;
    assert!(first.is_ok(), "expected initial token before cancel");

    // Now broadcast a blocking council verdict that should cancel the generation
    aura_backend::broadcast::broadcast_council(&store, &council_tx, None, "s1", "verdict", serde_json::json!({"final_state": "deny"}), Some(&gen_mgr));

    // Wait for end envelope
    let got_end = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(msg) = ai_rx.recv().await {
                if msg.contains("\"type\":\"end\"") && msg.contains(&gen_id) {
                    return true;
                }
            }
        }
    }).await;

    assert!(got_end.is_ok() && got_end.unwrap(), "expected end envelope after cancellation");
}
