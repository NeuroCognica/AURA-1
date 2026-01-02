use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

use crate::intent_classifier::Archetype;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    pub llm: LlmEndpoint,
    pub archetypes: HashMap<String, ArchetypeConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmEndpoint {
    pub endpoint: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchetypeConfig {
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub llm: LlmEndpoint,
    pub archetypes: HashMap<String, ArchetypeConfig>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

const REQUIRED_ARCHETYPES: [&str; 7] = [
    "Sentinel",
    "Architect",
    "Explorer",
    "Oracle",
    "Mentor",
    "Empath",
    "Jester",
];

// helper removed — mapping implemented directly where needed

impl OrchestratorConfig {
    pub fn load_from_str(s: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig = serde_json::from_str(s)?;

        // Validate archetype keys: exact set match
        for key in raw.archetypes.keys() {
            if !REQUIRED_ARCHETYPES.iter().any(|k| k == key) {
                return Err(ConfigError::Validation(format!("unknown archetype: {}", key)));
            }
        }

        for req in REQUIRED_ARCHETYPES.iter() {
            if !raw.archetypes.contains_key(*req) {
                return Err(ConfigError::Validation(format!("missing archetype: {}", req)));
            }
        }

        // Validate ranges and sentinel temp
        for (name, cfg) in raw.archetypes.iter() {
            if !(0.0..=1.0).contains(&cfg.temperature) {
                return Err(ConfigError::Validation(format!("temperature out of range for {}", name)));
            }
            if !(0.0..=1.0).contains(&cfg.top_p) {
                return Err(ConfigError::Validation(format!("top_p out of range for {}", name)));
            }
        }

        if let Some(s_cfg) = raw.archetypes.get("Sentinel") {
            if s_cfg.temperature != 0.0 {
                return Err(ConfigError::Validation("Sentinel temperature must be 0.0".to_string()));
            }
        }

        Ok(OrchestratorConfig {
            llm: raw.llm,
            archetypes: raw.archetypes,
        })
    }

    pub fn llm_endpoint(&self) -> &str {
        &self.llm.endpoint
    }

    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let s = fs::read_to_string(path)?;
        Self::load_from_str(&s)
    }

    pub fn model_for(&self, archetype: Archetype) -> &ArchetypeConfig {
        let key = match archetype {
            Archetype::Architect => "Architect",
            Archetype::Explorer => "Explorer",
            Archetype::Oracle => "Oracle",
            Archetype::Mentor => "Mentor",
            Archetype::Empath => "Empath",
            Archetype::Jester => "Jester",
        };

        &self.archetypes.get(key).expect("archetype present by validation")
    }
}
