use tempfile::TempDir;
use aura_backend::storage::RocksStore;
use tokio::sync::broadcast;

#[test]
fn council_interrupt_forwarding_minimal() {
    // Create a temp RocksDB store
    let td = TempDir::new().unwrap();
    let path = td.path().to_path_buf();
    let store = RocksStore::open(path).expect("open store");

    // Create council broadcast sender (string payloads)
    let (council_tx, mut council_rx) = broadcast::channel::<String>(16);

    // Build an InterruptPayload as JSON and a CouncilWsMsg wrapper
    let interrupt_payload = serde_json::json!({
        "kind": "halt_language",
        "scope": "generation",
        "reason": "require_consent",
        "requirements": null,
        "correlation": {"sid": "s123"}
    });

    let cmsg = aura_backend::council_verdict::CouncilWsMsg {
        kind: "interrupt".to_string(),
        session_id: "s123".to_string(),
        payload: interrupt_payload.clone(),
    };

    // Serialize and persist via broadcast_council helper
    aura_backend::broadcast::broadcast_council(&std::sync::Arc::new(store), &council_tx, "s123", "interrupt", interrupt_payload.clone(), None);

    // council_rx should receive the raw stored JSON string
    let received = council_rx.try_recv().expect("council message received");
    // It should parse to CouncilWsMsg and be an interrupt
    let parsed: aura_backend::council_verdict::CouncilWsMsg = serde_json::from_str(&received).expect("parse council msg");
    assert_eq!(parsed.kind, "interrupt");
    assert_eq!(parsed.session_id, "s123");
    assert!(parsed.payload.get("reason").is_some());

    // Now build the minimal notice using the library helper
    let minimal = aura_backend::broadcast::build_ai_interrupt_notice_from_council(&received).expect("should build minimal notice");
    let mval: serde_json::Value = serde_json::from_str(&minimal).expect("parse minimal");
    assert_eq!(mval.get("type").and_then(|v| v.as_str()).unwrap_or("") , "notice");
    assert_eq!(mval.get("notice").and_then(|v| v.as_str()).unwrap_or("") , "interrupt");

    // Ensure minimal payload does NOT include full requirements field
    let p = &mval["payload"];
    assert!(p.get("kind").is_some());
    assert!(p.get("scope").is_some());
    assert!(p.get("reason").is_some());
    // requirements should not be forwarded (it would be inside original payload)
    assert!(p.get("requirements").is_none());
}
