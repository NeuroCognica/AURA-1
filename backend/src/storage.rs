#![allow(dead_code)]

#[cfg(feature = "persistence")]
mod __storage_impl {
    use rocksdb::{checkpoint::Checkpoint, ColumnFamilyDescriptor, Options, WriteBatch, DB};
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct LogEntry {
        pub id: u64,
        pub timestamp_ms: i64,
        pub speaker: String,
        pub content: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Peak {
        pub height: u32,
        pub hash: Vec<u8>,
    }

    #[derive(Clone)]
    pub struct RocksStore {
        db: Arc<DB>,
        // CF names are: "logs", "state", "vectors"
    }

    impl RocksStore {
        pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
            let mut opts = Options::default();
            opts.create_if_missing(true);
            opts.create_missing_column_families(true);

            let cf_names = vec![
                ColumnFamilyDescriptor::new("default", Options::default()),
                ColumnFamilyDescriptor::new("logs", Options::default()),
                ColumnFamilyDescriptor::new("state", Options::default()),
                ColumnFamilyDescriptor::new("vectors", Options::default()),
            ];

            let db = DB::open_cf_descriptors(&opts, path, cf_names)?;
            Ok(Self { db: Arc::new(db) })
        }

        fn cf_handle(&self, name: &str) -> &rocksdb::ColumnFamily {
            self.db
                .cf_handle(name)
                .expect("column family should exist")
        }

        pub fn get_last_id(&self) -> anyhow::Result<u64> {
            let cf = self.cf_handle("state");
            match self.db.get_cf(cf, b"last_log_id")? {
                Some(v) if v.len() == 8 => Ok(u64::from_be_bytes(v.as_slice().try_into().unwrap())),
                _ => Ok(0),
            }
        }

        pub fn get_root(&self) -> anyhow::Result<Option<Vec<u8>>> {
            let cf = self.cf_handle("state");
            Ok(self.db.get_cf(cf, b"mmr_root")?)
        }

        pub fn get_log(&self, id: u64) -> anyhow::Result<Option<LogEntry>> {
            let cf = self.cf_handle("logs");
            let key = id.to_be_bytes();
            if let Some(v) = self.db.get_cf(cf, &key)? {
                let le: LogEntry = serde_json::from_slice(&v)?;
                Ok(Some(le))
            } else {
                Ok(None)
            }
        }

        /// Atomic append: writes log entry and updates MMR peaks + root in one WriteBatch.
        pub fn append_log_atomic(&self, speaker: &str, content: &str) -> anyhow::Result<u64> {
            let mut batch = WriteBatch::default();

            // load state
            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            let last_id = match self.db.get_cf(state_cf, b"last_log_id")? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            let new_id = last_id + 1;
            let entry = LogEntry {
                id: new_id,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                speaker: speaker.to_string(),
                content: content.to_string(),
            };

            let ser = serde_json::to_vec(&entry)?;

            // leaf hash
            let leaf_hash = blake3::hash(&ser).as_bytes().to_vec();

            // load peaks
            let mut peaks: Vec<Peak> = match self.db.get_cf(state_cf, b"mmr_peaks")? {
                Some(v) => serde_json::from_slice(&v)?,
                None => Vec::new(),
            };

            // create node
            let mut node = Peak {
                height: 0,
                hash: leaf_hash,
            };

            // merge peaks while heights equal
            while let Some(last) = peaks.last() {
                if last.height == node.height {
                    let left = peaks.pop().unwrap();
                    let mut merged = Vec::with_capacity(left.hash.len() + node.hash.len());
                    merged.extend_from_slice(&left.hash);
                    merged.extend_from_slice(&node.hash);
                    let parent = blake3::hash(&merged).as_bytes().to_vec();
                    node = Peak {
                        height: node.height + 1,
                        hash: parent,
                    };
                } else {
                    break;
                }
            }
            peaks.push(node.clone());

            // compute root: bagging peaks (hash of concatenated peak hashes)
            let mut concat = Vec::new();
            for p in &peaks {
                concat.extend_from_slice(&p.hash);
            }
            let root = blake3::hash(&concat).as_bytes().to_vec();

            // write to batch
            batch.put_cf(logs_cf, &new_id.to_be_bytes(), &ser);
            batch.put_cf(state_cf, b"last_log_id", &new_id.to_be_bytes());
            batch.put_cf(state_cf, b"mmr_peaks", serde_json::to_vec(&peaks)?.as_slice());
            batch.put_cf(state_cf, b"mmr_root", &root);

            // commit atomically
            self.db.write(batch)?;

            Ok(new_id)
        }

        /// Return a non-compact proof: the leaf hash and the current peaks (sufficient to recompute root).
        pub fn prove(&self, id: u64) -> anyhow::Result<Option<(Vec<u8>, Vec<Peak>)>> {
            let cf = self.cf_handle("logs");
            if let Some(v) = self.db.get_cf(cf, &id.to_be_bytes())? {
                let leaf_hash = blake3::hash(&v).as_bytes().to_vec();
                let state_cf = self.cf_handle("state");
                let peaks: Vec<Peak> = match self.db.get_cf(state_cf, b"mmr_peaks")? {
                    Some(b) => serde_json::from_slice(&b)?,
                    None => Vec::new(),
                };
                Ok(Some((leaf_hash, peaks)))
            } else {
                Ok(None)
            }
        }

        pub fn create_snapshot(&self, dst: &Path) -> anyhow::Result<()> {
                // If destination exists, remove it so Checkpoint can create it fresh.
                if dst.exists() {
                    std::fs::remove_dir_all(dst)?;
                }
                let cp = Checkpoint::new(self.db.as_ref())?;
                cp.create_checkpoint(dst)?;
                Ok(())
        }
    }
}

// Re-export items at crate level when feature is enabled
#[cfg(feature = "persistence")]
pub use __storage_impl::{LogEntry, Peak, RocksStore};
