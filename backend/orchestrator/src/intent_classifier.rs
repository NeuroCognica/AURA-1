use serde::{Deserialize, Serialize};
use thiserror::Error;

// keep reference to backend types if needed later

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IntentClassification {
    pub primary_archetype: Archetype,
    pub secondary_archetypes: Vec<Archetype>,
    pub task_summary: String,
    pub is_authority_query: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum Archetype {
    Architect,
    Explorer,
    Oracle,
    Mentor,
    Empath,
    Jester,
}

#[derive(Debug, Error)]
pub enum ClassifierError {
    #[error("invalid classifier output schema")]
    InvalidSchema,

    #[error("classifier transport error")]
    Transport,

    #[error("classifier unavailable")]
    Unavailable,
}

/// Schema-locked, deterministic classifier scaffold.
/// For now this only validates and parses the provided JSON.
pub fn classify_intent(raw_json: &str) -> Result<IntentClassification, ClassifierError> {
    serde_json::from_str::<IntentClassification>(raw_json)
        .map_err(|_| ClassifierError::InvalidSchema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_schema() {
        let json = r#"
        {
            "primary_archetype": "Architect",
            "secondary_archetypes": ["Oracle"],
            "task_summary": "Design a system architecture",
            "is_authority_query": false
        }
        "#;

        let out = classify_intent(json).unwrap();
        assert_eq!(out.primary_archetype, Archetype::Architect);
    }

    #[test]
    fn rejects_extra_fields() {
        let json = r#"
        {
            "primary_archetype": "Architect",
            "secondary_archetypes": [],
            "task_summary": "Test",
            "is_authority_query": false,
            "extra": "nope"
        }
        "#;

        assert!(classify_intent(json).is_err());
    }

    #[test]
    fn rejects_invalid_archetype() {
        let json = r#"
        {
            "primary_archetype": "GodMode",
            "secondary_archetypes": [],
            "task_summary": "lol",
            "is_authority_query": false
        }
        "#;

        assert!(classify_intent(json).is_err());
    }
}
