#![cfg(feature = "persistence")]

use aura_backend::rocksdb_store::RocksDBStore;
use tempfile::TempDir;

#[test]
fn mmr_proof_verification_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");
    let path = tmp.path();

    let store = RocksDBStore::open(path).expect("open db");

    // append some logs
    for i in 0..5 {
        let content = format!("entry {}", i);
        let id = store.append_log_atomic("tester", &content).expect("append");
        assert!(id > 0);
    }

    // pick an id to prove
    let last = store.get_last_id().expect("last id");
    let (leaf, peaks) = store
        .prove(last)
        .expect("prove returned")
        .expect("proof exists");

    // recompute root from peaks (bagging)
    let mut concat = Vec::new();
    for p in &peaks {
        concat.extend_from_slice(&p.hash);
    }
    let recomputed_root = blake3::hash(&concat).as_bytes().to_vec();

    // get stored root
    let stored = store.get_root().expect("get root").expect("root exists");

    assert_eq!(
        recomputed_root, stored,
        "recomputed root matches stored mmr_root"
    );

    // Verify leaf is consistent with stored log bytes
    let le = store.get_log(last).expect("get log").expect("exists");
    let ser = serde_json::to_vec(&le).expect("serialize log");
    let leaf_expected = blake3::hash(&ser).as_bytes().to_vec();
    assert_eq!(leaf_expected, leaf, "leaf hash matches log content hash");
}
