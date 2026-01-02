#![cfg(feature = "persistence")]

use aura_backend::storage::RocksStore;
use tempfile::TempDir;

#[test]
fn storage_append_and_load_session_chat_and_meta() {
    let td = TempDir::new().unwrap();
    let path = td.path().to_path_buf();
    let store = RocksStore::open(path).expect("open store");

    // append chat messages
    let sid = "test-session";
    let _ = store
        .append_chat_msg(sid, "user", "hello world")
        .expect("append chat");
    let _ = store
        .append_chat_msg(sid, "assistant", "hi")
        .expect("append chat");

    let recent = store.load_recent_chat(sid, 10).expect("load recent");
    assert!(recent.len() >= 2);

    // append session metadata
    let _ = store
        .append_session_meta(sid, "{\"mode\":\"code\"}")
        .expect("append meta");
    let meta = store.load_recent_session_meta(sid, 1).expect("load meta");
    assert_eq!(meta.len(), 1);

    // append a general log entry and validate prove/root
    let id = store
        .append_log_atomic("system", "system event")
        .expect("append log");
    let proof = store.prove(id).expect("prove").expect("some proof");
    let root = store.get_root().expect("get root").expect("root present");
    // root should be non-empty
    assert!(!root.is_empty());
    // leaf hash should be non-empty
    assert!(!proof.0.is_empty());
}
