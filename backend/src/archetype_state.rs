use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeState {
    pub active: ArchetypeId,
    pub transition_mode: TransitionMode,
    pub activation_timestamp: DateTime<Utc>,
    pub previous: Option<ArchetypeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionMode {
    Instant,
    Ritual(u32), // milliseconds
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArchetypeId {
    Architect,
    Sentinel,
    Jester,
    Witness,
    Mentor,
    Empath,
    Oracle,
    Explorer,
}

impl ArchetypeId {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "architect" => Some(ArchetypeId::Architect),
            "sentinel" => Some(ArchetypeId::Sentinel),
            "jester" => Some(ArchetypeId::Jester),
            "witness" => Some(ArchetypeId::Witness),
            "mentor" => Some(ArchetypeId::Mentor),
            "empath" => Some(ArchetypeId::Empath),
            "oracle" => Some(ArchetypeId::Oracle),
            "explorer" => Some(ArchetypeId::Explorer),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ArchetypeId::Architect => "architect",
            ArchetypeId::Sentinel => "sentinel",
            ArchetypeId::Jester => "jester",
            ArchetypeId::Witness => "witness",
            ArchetypeId::Mentor => "mentor",
            ArchetypeId::Empath => "empath",
            ArchetypeId::Oracle => "oracle",
            ArchetypeId::Explorer => "explorer",
        }
    }
}
