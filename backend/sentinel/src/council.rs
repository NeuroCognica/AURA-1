use serde::{Deserialize, Serialize};

/// Minimal CouncilMsg scaffold for sentinel crate tests and prompt building.
#[derive(Debug, Serialize, Deserialize)]
pub enum CouncilMsg {
    Query { id: u64, content: String },
    Response { archetype: String, analysis: String },
}
