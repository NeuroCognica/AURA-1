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
            self.db.cf_handle(name).expect("column family should exist")
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
            batch.put_cf(
                state_cf,
                b"mmr_peaks",
                serde_json::to_vec(&peaks)?.as_slice(),
            );
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

        /// Generic state accessor: read arbitrary bytes from the `state` column family.
        pub fn get_bytes(&self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
            let cf = self.cf_handle("state");
            Ok(self.db.get_cf(cf, key)?)
        }

        /// Generic state writer: write arbitrary bytes to the `state` column family.
        pub fn put_bytes(&self, key: &[u8], val: &[u8]) -> anyhow::Result<()> {
            let cf = self.cf_handle("state");
            self.db.put_cf(cf, key, val)?;
            Ok(())
        }

        /// Append a chat message to a named session. Returns the session sequence number.
        pub fn append_chat_msg(
            &self,
            session_id: &str,
            speaker: &str,
            content: &str,
        ) -> anyhow::Result<u64> {
            let mut batch = WriteBatch::default();

            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            // key for last seq for this session
            let last_key = format!("sess_last:{}", session_id);

            let last_seq = match self.db.get_cf(state_cf, last_key.as_bytes())? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            let new_seq = last_seq + 1;

            let entry = LogEntry {
                id: new_seq,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                speaker: speaker.to_string(),
                content: content.to_string(),
            };

            let ser = serde_json::to_vec(&entry)?;

            // composite key: sess:<session_id>:<seq padded>
            let key = format!("sess:{}:{:020}", session_id, new_seq);

            batch.put_cf(logs_cf, key.as_bytes(), &ser);
            batch.put_cf(state_cf, last_key.as_bytes(), &new_seq.to_be_bytes());

            // commit batch
            self.db.write(batch)?;

            Ok(new_seq)
        }

        /// Load the most recent `limit` chat messages for the given session (ascending order).
        pub fn load_recent_chat(
            &self,
            session_id: &str,
            limit: usize,
        ) -> anyhow::Result<Vec<LogEntry>> {
            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            let last_key = format!("sess_last:{}", session_id);
            let last_seq = match self.db.get_cf(state_cf, last_key.as_bytes())? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            if last_seq == 0 {
                return Ok(Vec::new());
            }

            let mut out = Vec::new();
            let start = if last_seq > limit as u64 {
                last_seq - (limit as u64) + 1
            } else {
                1
            };
            for seq in start..=last_seq {
                let key = format!("sess:{}:{:020}", session_id, seq);
                if let Some(v) = self.db.get_cf(logs_cf, key.as_bytes())? {
                    let le: LogEntry = serde_json::from_slice(&v)?;
                    out.push(le);
                }
            }

            Ok(out)
        }

        /// Append a typed council envelope atomically.
        ///
        /// This function computes a new per-session sequence number,
        /// invokes the provided closure with that sequence to obtain
        /// the serialized envelope bytes, and atomically writes the
        /// envelope under the `logs` column family and updates the
        /// session's last seq in the `state` column family.
        pub fn append_council_envelope_with<F>(&self, session_id: &str, f: F) -> anyhow::Result<u64>
        where
            F: FnOnce(u64) -> Vec<u8>,
        {
            let mut batch = WriteBatch::default();

            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            // key for last seq for council for this session
            let last_key = format!("sess_council_last:{}", session_id);

            let last_seq = match self.db.get_cf(state_cf, last_key.as_bytes())? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            let new_seq = last_seq + 1;

            // Let caller produce the serialized envelope bytes now that we have seq
            let ser = f(new_seq);

            // composite key: sess_council:{session_id}:{seq padded}
            let key = format!("sess_council:{}:{:020}", session_id, new_seq);

            batch.put_cf(logs_cf, key.as_bytes(), &ser);
            batch.put_cf(state_cf, last_key.as_bytes(), &new_seq.to_be_bytes());

            // commit batch
            self.db.write(batch)?;

            Ok(new_seq)
        }

        /// Retrieve a persisted council envelope for a given session and sequence.
        pub fn get_council_envelope(
            &self,
            session_id: &str,
            seq: u64,
        ) -> anyhow::Result<Option<Vec<u8>>> {
            let logs_cf = self.cf_handle("logs");
            let key = format!("sess_council:{}:{:020}", session_id, seq);
            Ok(self.db.get_cf(logs_cf, key.as_bytes())?)
        }

        /// Load a range of persisted council envelopes (inclusive).
        pub fn load_council_range(
            &self,
            session_id: &str,
            from_seq: u64,
            to_seq: u64,
        ) -> anyhow::Result<Vec<Vec<u8>>> {
            let mut out = Vec::new();
            let logs_cf = self.cf_handle("logs");
            for seq in from_seq..=to_seq {
                let key = format!("sess_council:{}:{:020}", session_id, seq);
                if let Some(v) = self.db.get_cf(logs_cf, key.as_bytes())? {
                    out.push(v);
                } else {
                    // missing envelope in range; stop early
                    break;
                }
            }
            Ok(out)
        }

        /// Append a session metadata entry (append-only). Returns sequence number for the session.
        pub fn append_session_meta(&self, session_id: &str, content: &str) -> anyhow::Result<u64> {
            let mut batch = WriteBatch::default();

            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            // key for last seq for this session metadata
            let last_key = format!("sess_meta_last:{}", session_id);

            let last_seq = match self.db.get_cf(state_cf, last_key.as_bytes())? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            let new_seq = last_seq + 1;

            let entry = LogEntry {
                id: new_seq,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                speaker: "meta".to_string(),
                content: content.to_string(),
            };

            let ser = serde_json::to_vec(&entry)?;

            // composite key: sess_meta:{session_id}:{seq padded}
            let key = format!("sess_meta:{}:{:020}", session_id, new_seq);

            batch.put_cf(logs_cf, key.as_bytes(), &ser);
            batch.put_cf(state_cf, last_key.as_bytes(), &new_seq.to_be_bytes());

            self.db.write(batch)?;

            Ok(new_seq)
        }

        /// Load the most recent `limit` session metadata entries for the given session (ascending order).
        pub fn load_recent_session_meta(
            &self,
            session_id: &str,
            limit: usize,
        ) -> anyhow::Result<Vec<LogEntry>> {
            let state_cf = self.cf_handle("state");
            let logs_cf = self.cf_handle("logs");

            let last_key = format!("sess_meta_last:{}", session_id);
            let last_seq = match self.db.get_cf(state_cf, last_key.as_bytes())? {
                Some(v) if v.len() == 8 => u64::from_be_bytes(v.as_slice().try_into().unwrap()),
                _ => 0u64,
            };

            if last_seq == 0 {
                return Ok(Vec::new());
            }

            let mut out = Vec::new();
            let start = if last_seq > limit as u64 {
                last_seq - (limit as u64) + 1
            } else {
                1
            };
            for seq in start..=last_seq {
                let key = format!("sess_meta:{}:{:020}", session_id, seq);
                if let Some(v) = self.db.get_cf(logs_cf, key.as_bytes())? {
                    let le: LogEntry = serde_json::from_slice(&v)?;
                    out.push(le);
                }
            }

            Ok(out)
        }
    }
}

// Re-export items at crate level when feature is enabled
#[cfg(feature = "persistence")]
pub use __storage_impl::RocksStore;
