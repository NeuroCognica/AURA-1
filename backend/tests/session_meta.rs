use aura_backend::storage::RocksStore;
use tempfile::TempDir;

#[test]
fn append_and_load_session_meta() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path();

    let store = RocksStore::open(path).expect("open db");

    let sid = "sess-meta-test";

    // no meta yet
    let before = store.load_recent_session_meta(sid, 10).expect("load");
    assert!(before.is_empty());

    // append 3 metadata entries
    for i in 0..3u64 {
        let content = format!("{{\"mode\": \"Mode{}\"}}", i);
        let seq = store.append_session_meta(sid, &content).expect("append meta");
        assert_eq!(seq, i + 1);
    }

    // load recent 2 entries
    let recent = store.load_recent_session_meta(sid, 2).expect("load recent");
    assert_eq!(recent.len(), 2);
    assert!(recent[0].content.contains("Mode2") || recent[1].content.contains("Mode2"));
}
