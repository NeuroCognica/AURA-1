use aura_backend::storage::RocksStore;
use tempfile::TempDir;

#[test]
fn append_and_prove_and_snapshot() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path();

    let store = RocksStore::open(path).expect("open db");

    // append a few logs
    for i in 0..3 {
        let content = format!("hello {}", i);
        let id = store.append_log_atomic("tester", &content).expect("append");
        assert!(id > 0);
        let got = store.get_log(id).expect("get log").expect("exists");
        assert_eq!(got.content, content);
    }

    // prove last
    let last = store.get_last_id().expect("last id");
    let proof = store.prove(last).expect("prove").expect("proof exists");
    assert!(!proof.0.is_empty());

    // snapshot
    let snap = path.join("checkpoint");
    std::fs::create_dir_all(&snap).unwrap();
    store.create_snapshot(&snap).expect("snapshot");
    assert!(snap.exists());
}
