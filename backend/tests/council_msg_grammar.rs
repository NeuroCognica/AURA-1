use aura_backend::council_verdict::*;
use serde_json::json;

#[test]
fn proposed_action_roundtrip() {
    let pa = ProposedAction {
        action_id: "a1".to_string(),
        title: "Test Action".to_string(),
        description: Some("do something".to_string()),
        metadata: None,
    };

    let msg = CouncilMsg::ProposedAction(pa);
    let s1 = serde_json::to_string(&msg).expect("serialize");
    let msg2: CouncilMsg = serde_json::from_str(&s1).expect("deserialize");
    let s2 = serde_json::to_string(&msg2).expect("serialize2");
    assert_eq!(s1, s2, "ProposedAction should round-trip exactly");
}

#[test]
fn verdict_roundtrip() {
    let v = CouncilVerdict {
        verdict_id: VerdictId("v1".to_string()),
        session_id: SessionId("s1".to_string()),
        timestamp_ms: 1u128,
        active_archetype: ArchetypeId("sentinel".to_string()),
        capability: Capability::General,
        sentinel_kind: SentinelDecisionKind::Deny,
        sentinel_constraints: vec![],
        sentinel_message: "denied by test".to_string(),
        consent: None,
        dissent: std::collections::BTreeMap::new(),
        alchemist_options: vec![],
        final_state: FinalState::Blocked,
        next_required: NextRequired::None,
    };

    let msg = CouncilMsg::Verdict(v);
    let s1 = serde_json::to_string(&msg).expect("serialize verdict");
    let msg2: CouncilMsg = serde_json::from_str(&s1).expect("deserialize verdict");
    let s2 = serde_json::to_string(&msg2).expect("serialize2 verdict");
    assert_eq!(s1, s2, "Verdict should round-trip exactly");
}

#[test]
fn envelope_mapping_and_replay() {
    let pa = ProposedAction {
        action_id: "a2".to_string(),
        title: "Map Test".to_string(),
        description: None,
        metadata: Some(json!({"k":"v"})),
    };

    let msg = CouncilMsg::ProposedAction(pa.clone());
    let env = make_council_envelope("sid", Some("vid".to_string()), 42, msg.clone());

    // msg_type should serialize to the snake_case name
    let mt = serde_json::to_value(&env.msg_type).expect("msg_type to value");
    assert_eq!(mt, json!("proposed_action"));

    // payload should equal the ProposedAction as JSON
    let payload = env.payload.clone();
    let expected_payload = serde_json::to_value(&pa).expect("pa to value");
    assert_eq!(payload, expected_payload);

    // replay: serialize envelope -> deserialize -> serialize -> compare
    let s = serde_json::to_string(&env).expect("serialize envelope");
    let env2: CouncilEnvelope = serde_json::from_str(&s).expect("deserialize envelope");
    let s2 = serde_json::to_string(&env2).expect("serialize envelope2");
    assert_eq!(s, s2, "Envelope should be byte-stable across replay");
}

#[test]
fn cognition_cannot_masquerade_as_verdict() {
    // create a ProposedAction payload but label the envelope as a verdict
    let pa = ProposedAction {
        action_id: "a3".to_string(),
        title: "Masquerade".to_string(),
        description: None,
        metadata: None,
    };

    let payload = serde_json::to_value(&pa).expect("pa to value");
    let envelope_json = json!({
        "seq": 7,
        "type": "verdict",
        "ts_ms": 12345u64,
        "sid": "smasq",
        "vid": null,
        "payload": payload,
    });

    // parse as CouncilEnvelope (payload is serde_json::Value)
    let env: CouncilEnvelope = serde_json::from_value(envelope_json).expect("parse envelope");

    // Attempting to interpret the payload as a real CouncilVerdict must fail
    let maybe_verdict: Result<CouncilVerdict, _> = serde_json::from_value(env.payload);
    assert!(maybe_verdict.is_err(), "Cognition payload must not deserialize as a Verdict");
}
