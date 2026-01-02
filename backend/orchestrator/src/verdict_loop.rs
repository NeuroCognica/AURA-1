use async_trait::async_trait;
use thiserror::Error;
use serde_json::Value;

use crate::intent_classifier::Archetype;
use aura_backend::council_verdict::{CouncilVerdict, FinalState, ProposedAction as SharedProposedAction};

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Approve,
    Deny,
    Refuse,
}

#[derive(Debug, Error, Clone)]
pub enum VerdictError {
    #[error("transport error")]
    Transport,
    #[error("timeout or no response")]
    Timeout,
    #[error("invalid response")]
    InvalidResponse,
}

/// A minimal client interface to the authority spine used by the Orchestrator.
#[async_trait]
pub trait AuthorityClient: Send + Sync {
    async fn submit_proposed_action(
        &self,
        proposal: SharedProposedAction,
    ) -> Result<CouncilVerdict, VerdictError>;
}

/// Orchestrator's non-authoritative proposed action shape.
pub struct ProposedAction {
    pub summary: String,
    pub archetype: Archetype,
    pub payload: Value,
}

/// Map the shared `CouncilVerdict` to a local `Verdict` enum.
fn map_council_verdict(v: &CouncilVerdict) -> Verdict {
    match v.final_state {
        FinalState::Allowed | FinalState::AllowedWithWarning => Verdict::Approve,
        FinalState::Blocked => Verdict::Deny,
        _ => Verdict::Refuse,
    }
}

/// Request a verdict from the Authority. This function is mechanical and
/// accepts only Approve/Deny/Refuse as valid outcomes. Any unexpected
/// result is treated as `Refuse`.
pub async fn request_verdict(
    action: ProposedAction,
    authority: &dyn AuthorityClient,
) -> Result<Verdict, VerdictError> {
    // Convert to shared ProposedAction shape used on the authority spine.
    let shared = SharedProposedAction {
        action_id: uuid::Uuid::new_v4().to_string(),
        title: action.summary.clone(),
        description: None,
        metadata: Some(action.payload),
    };

    let verdict = authority.submit_proposed_action(shared).await?;
    Ok(map_council_verdict(&verdict))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_backend::council_verdict::{CouncilVerdict, VerdictId, SessionId, ArchetypeId, Capability, SentinelDecisionKind, NextRequired, FinalState};
    use std::collections::BTreeMap;

    struct MockAuthority {
        verdict: Option<CouncilVerdict>,
        err: Option<VerdictError>,
    }

    #[async_trait]
    impl AuthorityClient for MockAuthority {
        async fn submit_proposed_action(&self, _proposal: SharedProposedAction) -> Result<CouncilVerdict, VerdictError> {
            if let Some(e) = &self.err {
                return Err(e.clone());
            }
            if let Some(v) = &self.verdict {
                return Ok(v.clone());
            }
            Err(VerdictError::Timeout)
        }
    }

    fn make_council_verdict(state: FinalState) -> CouncilVerdict {
        CouncilVerdict {
            verdict_id: VerdictId("vtest".to_string()),
            session_id: SessionId("stest".to_string()),
            timestamp_ms: 0u128,
            active_archetype: ArchetypeId("sentinel".to_string()),
            capability: Capability::General,
            sentinel_kind: SentinelDecisionKind::Deny,
            sentinel_constraints: vec![],
            sentinel_message: "test".to_string(),
            consent: None,
            dissent: BTreeMap::new(),
            alchemist_options: vec![],
            final_state: state,
            next_required: NextRequired::None,
        }
    }

    #[tokio::test]
    async fn approve_allows_execution() {
        let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::Allowed)), err: None };
        let action = ProposedAction { summary: "s".into(), archetype: Archetype::Explorer, payload: serde_json::json!({}) };
        let res = request_verdict(action, &ma).await.unwrap();
        assert_eq!(res, Verdict::Approve);
    }

    #[tokio::test]
    async fn deny_blocks_execution() {
        let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::Blocked)), err: None };
        let action = ProposedAction { summary: "s".into(), archetype: Archetype::Explorer, payload: serde_json::json!({}) };
        let res = request_verdict(action, &ma).await.unwrap();
        assert_eq!(res, Verdict::Deny);
    }

    #[tokio::test]
    async fn refuse_on_other_states() {
        let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::RequireConsent)), err: None };
        let action = ProposedAction { summary: "s".into(), archetype: Archetype::Explorer, payload: serde_json::json!({}) };
        let res = request_verdict(action, &ma).await.unwrap();
        assert_eq!(res, Verdict::Refuse);
    }

    #[tokio::test]
    async fn invalid_response_treated_as_refuse() {
        let ma = MockAuthority { verdict: None, err: Some(VerdictError::Transport) };
        let action = ProposedAction { summary: "s".into(), archetype: Archetype::Explorer, payload: serde_json::json!({}) };
        let r = request_verdict(action, &ma).await;
        assert!(r.is_err());
    }
}
