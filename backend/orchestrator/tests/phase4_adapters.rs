use orchestrator::execution::types::*;
use orchestrator::execution::adapters::deterministic::DeterministicAdapter;
use orchestrator::execution::adapters::AdapterError;
use orchestrator::execution::adapters::ArchetypeAdapter;
use std::collections::BTreeMap;

#[tokio::test]
async fn deterministic_adapter_returns_explanation_and_no_proposals() {
    let adapter = DeterministicAdapter::new(ArchetypeId::Technician);
    let input = AdapterInput {
        request_id: "req-1".to_string(),
        user_text: "do something".to_string(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let out = adapter.run(input).await.expect("adapter should succeed");
    assert_eq!(out.request_id, "req-1");
    assert_eq!(out.proposals.len(), 0);
    assert!(out.explanation.contains("Deterministic"));
}

#[tokio::test]
async fn deterministic_adapter_rejects_empty_request_id() {
    let adapter = DeterministicAdapter::new(ArchetypeId::Technician);
    let input = AdapterInput {
        request_id: "".to_string(),
        user_text: "do something".to_string(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let res = adapter.run(input).await;
    match res {
        Err(AdapterError::InvalidInput(_)) => {}
        other => panic!("expected InvalidInput, got {:?}", other),
    }
}
