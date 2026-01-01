use aura_backend::council_verdict::{CouncilVerdict, VerdictId, SessionId, ArchetypeId, Capability, SentinelDecisionKind, NextRequired, FinalState};
use orchestrator::execution::runner::{run_proposal, ExecutionMode, ExecutionError};
use orchestrator::verdict_loop::{AuthorityClient, VerdictError, ProposedAction};
use std::collections::BTreeMap;

struct MockAuthority {
    verdict: Option<CouncilVerdict>,
    err: Option<VerdictError>,
}

#[async_trait::async_trait]
impl AuthorityClient for MockAuthority {
    async fn submit_proposed_action(&self, _proposal: aura_backend::council_verdict::ProposedAction) -> Result<CouncilVerdict, VerdictError> {
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
async fn deny_blocks_execution() {
    let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::Blocked)), err: None };
    let action = ProposedAction { summary: "s".into(), archetype: orchestrator::intent_classifier::Archetype::Explorer, payload: serde_json::json!({}) };
    let res = run_proposal(ExecutionMode::DryRun, &ma, action).await;
    match res {
        Err(ExecutionError::Denied(_)) => {}
        other => panic!("expected Denied, got {:?}", other),
    }
}

#[tokio::test]
async fn approve_dryrun_logs_but_no_effects() {
    let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::Allowed)), err: None };
    let action = ProposedAction { summary: "s".into(), archetype: orchestrator::intent_classifier::Archetype::Explorer, payload: serde_json::json!({}) };
    let res = run_proposal(ExecutionMode::DryRun, &ma, action).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn refuse_blocks_execution() {
    let ma = MockAuthority { verdict: Some(make_council_verdict(FinalState::RequireConsent)), err: None };
    let action = ProposedAction { summary: "s".into(), archetype: orchestrator::intent_classifier::Archetype::Explorer, payload: serde_json::json!({}) };
    let res = run_proposal(ExecutionMode::DryRun, &ma, action).await;
    match res {
        Err(ExecutionError::Refused(_)) => {}
        other => panic!("expected Refused, got {:?}", other),
    }
}
