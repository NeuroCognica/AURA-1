use anyhow::Result;
use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerRecord {
    pub probe_id: String,
    pub answer: String,
    pub ts_ms: i64,
}

pub struct Store {
    db: DB,
}

impl Store {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }

    fn counter_key(session_id: &str) -> String {
        format!("session:{}:counter", session_id)
    }

    fn probe_map_key(session_id: &str, probe_id: &str) -> String {
        format!("session:{}:probe:{}", session_id, probe_id)
    }

    fn answer_key(session_id: &str, idx: u64) -> String {
        format!("session:{}:answer:{:020}", session_id, idx)
    }

    /// Insert or update an answer for a probe. If the probe has been answered before,
    /// overwrite the existing record. Otherwise append and return new index.
    pub fn upsert_answer(&self, session_id: &str, probe_id: &str, answer: &str) -> Result<u64> {
        let pmap = Self::probe_map_key(session_id, probe_id);

        // If mapping exists, overwrite at that index
        if let Some(idx_bytes) = self.db.get(pmap.as_bytes())? {
            if idx_bytes.len() == 8 {
                let idx = u64::from_le_bytes(idx_bytes.try_into().unwrap());
                let rec = AnswerRecord {
                    probe_id: probe_id.to_string(),
                    answer: answer.to_string(),
                    ts_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64,
                };
                let key = Self::answer_key(session_id, idx);
                let val = serde_json::to_vec(&rec)?;
                self.db.put(key.as_bytes(), &val)?;
                return Ok(idx);
            }
        }

        // otherwise append as new
        let ck = Self::counter_key(session_id);
        let mut idx: u64 = 0;
        if let Some(v) = self.db.get(ck.as_bytes())? {
            if v.len() == 8 {
                idx = u64::from_le_bytes(v.try_into().unwrap());
            }
        }
        idx += 1;
        self.db.put(ck.as_bytes(), &idx.to_le_bytes())?;

        let rec = AnswerRecord {
            probe_id: probe_id.to_string(),
            answer: answer.to_string(),
            ts_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64,
        };

        let key = Self::answer_key(session_id, idx);
        let val = serde_json::to_vec(&rec)?;
        self.db.put(key.as_bytes(), &val)?;

        // record probe->idx mapping
        self.db.put(pmap.as_bytes(), &idx.to_le_bytes())?;
        Ok(idx)
    }

    pub fn get_session_answers(&self, session_id: &str) -> Result<Vec<AnswerRecord>> {
        let prefix = format!("session:{}:answer:", session_id);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        let mut out = Vec::new();
        for item in iter {
            let (_k, v) = item?;
            let rec: AnswerRecord = serde_json::from_slice(&v)?;
            out.push(rec);
        }
        Ok(out)
    }

    pub fn answered_count(&self, session_id: &str) -> Result<u64> {
        let ck = Self::counter_key(session_id);
        if let Some(v) = self.db.get(ck.as_bytes())? {
            if v.len() == 8 {
                return Ok(u64::from_le_bytes(v.try_into().unwrap()));
            }
        }
        Ok(0)
    }
}
