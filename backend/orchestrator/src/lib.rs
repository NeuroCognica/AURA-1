pub mod intent_classifier;
pub mod intent_stratification;
pub mod router;
pub mod verdict_loop;
pub mod architect;
pub mod deliberation;
pub mod config;
pub mod execution;
pub mod llm;
pub mod prompts;
pub mod constitutional;
pub mod qsic;

use aura_backend::council_verdict::*;
use serde::{Deserialize, Serialize};

/// Minimal orchestrator scaffold. No Sentinel logic, no persistence.
/// This crate only speaks the CouncilMsg typed contract.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub name: String,
}

pub fn version() -> &'static str {
    "0.1.0"
}

/// Example API: create a `ProposedAction` wrapped as a `CouncilMsg`.
pub fn make_proposed_action(action_id: &str, title: &str, description: Option<&str>) -> CouncilMsg {
    let pa = aura_backend::council_verdict::ProposedAction {
        action_id: action_id.to_string(),
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        metadata: None,
    };
    aura_backend::council_verdict::CouncilMsg::ProposedAction(pa)
}

pub use architect::*;
pub use deliberation::*;
pub use config::*;
pub use execution::*;
