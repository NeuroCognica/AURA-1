use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::intent_classifier::Archetype;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterInput {
    pub request_id: String,
    pub user_text: String,
    pub context: BTreeMap<String, String>,
    pub archetype: Archetype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArchetypeId {
    Architect,
    Sentinel,
    Empath,
    Technician,
    Historian,
    Oracle,
    Jester,
    Explorer,
    Mentor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterOutput {
    pub request_id: String,
    pub archetype: ArchetypeId,
    pub proposals: Vec<ProposedAction>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedAction {
    pub action_type: String,
    pub payload: serde_json::Value,
    pub risk: RiskLevel,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}
