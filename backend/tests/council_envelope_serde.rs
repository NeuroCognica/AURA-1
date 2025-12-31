use aura_backend::council_verdict::{CouncilMsg, CouncilVerdict, make_council_envelope};
use serde_json::Value;

#[test]
fn council_envelope_serializes_to_contract_shape() {
    // Build a minimal verdict payload to exercise serialization
    let verdict = CouncilVerdict {
        verdict_id: aura_backend::council_verdict::VerdictId("v-1".to_string()),
        session_id: aura_backend::council_verdict::SessionId("s-1".to_string()),
        timestamp_ms: 0,
        active_archetype: aura_backend::council_verdict::ArchetypeId("sentinel".to_string()),
        capability: aura_backend::council_verdict::Capability::General,
        sentinel_kind: aura_backend::council_verdict::SentinelDecisionKind::Allow,
        sentinel_constraints: vec![],
        sentinel_message: "ok".to_string(),
        consent: None,
        dissent: std::collections::BTreeMap::new(),
        alchemist_options: vec![],
        final_state: aura_backend::council_verdict::FinalState::Allowed,
        next_required: aura_backend::council_verdict::NextRequired::None,
    };

    let msg = CouncilMsg::Verdict(verdict);
    let env = make_council_envelope("s-1", None, 42, msg);
    let s = serde_json::to_string(&env).expect("serialize envelope");

    // Basic sanity checks on the serialized shape
    let v: Value = serde_json::from_str(&s).expect("parse json");
    assert_eq!(v.get("seq").and_then(|x| x.as_u64()), Some(42));
    assert!(v.get("type").is_some());
    assert_eq!(v.get("sid").and_then(|x| x.as_str()), Some("s-1"));
    assert!(v.get("ts_ms").is_some());
    assert!(v.get("payload").is_some());
}
