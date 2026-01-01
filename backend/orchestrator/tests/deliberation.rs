use orchestrator::deliberation::{DeliberationManager, Decision, DecisionOption};
use orchestrator::intent_classifier::Archetype;

#[tokio::test]
async fn deliberation_flow_create_wait_submit() {
    let manager = DeliberationManager::new();

    let option = DecisionOption { id: uuid::Uuid::new_v4(), label: "A".to_string() };

    let (d, handle) = manager
        .create_deliberation(
            uuid::Uuid::new_v4(),
            Archetype::Architect,
            "Test deliberation".to_string(),
            vec![option.clone()],
            Some(option.id),
            vec![],
        )
        .await;

    let id = d.id;

    let waiter = tokio::spawn(async move { handle.wait_for_decision().await });

    let ok = manager
        .submit_decision(id, Decision { choice_id: option.id, notes: None })
        .await;

    assert!(ok);

    let result = waiter.await.unwrap();
    assert_eq!(result.unwrap().choice_id, option.id);
}
