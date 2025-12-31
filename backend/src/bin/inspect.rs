use anyhow::Result;
use serde_json::to_string_pretty;
use std::path::PathBuf;

fn main() -> Result<()> {
    let path = PathBuf::from("data/rocksdb");
    println!("Opening RocksDB at {:?}", path);
    let store = aura_backend::storage::RocksStore::open(path)?;
    let session = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test123".to_string());
    let msgs = store.load_recent_chat(&session, 200)?;
    println!("Session {} => {} messages", session, msgs.len());
    println!("{}", to_string_pretty(&msgs)?);
    Ok(())
}
