use crate::store::{AnswerRecord, Store};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub model: String,
    pub request_id: String,
    pub timestamp: String,
    pub backend: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MirrorbornProfile {
    pub session_id: String,
    pub answers: Vec<AnswerRecord>,
    pub provenance: Provenance,
}

pub fn make_provenance(model: &str, backend: &str) -> Provenance {
    Provenance {
        model: model.to_string(),
        request_id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        backend: backend.to_string(),
    }
}

pub fn generate_profile(store: &Store, session_id: &str, artifacts_dir: &Path) -> Result<PathBuf> {
    let answers = store.get_session_answers(session_id)?;

    let profile = MirrorbornProfile {
        session_id: session_id.to_string(),
        answers,
        provenance: make_provenance("mirrorborn-v1", "aura_quiz_service"),
    };

    // ensure artifacts dir
    std::fs::create_dir_all(artifacts_dir)?;

    let filename = format!("profile_{}.json", session_id);
    let tmp = artifacts_dir.join(format!("{}.tmp", &filename));
    let finalp = artifacts_dir.join(&filename);

    let mut f = File::create(&tmp)?;
    let json = serde_json::to_vec_pretty(&profile)?;
    f.write_all(&json)?;
    f.flush()?;
    drop(f);

    std::fs::rename(&tmp, &finalp)?;
    Ok(finalp)
}
