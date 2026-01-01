use orchestrator::architect::{plan_task, TaskPlan};
use orchestrator::intent_classifier::Archetype;

#[tokio::test]
async fn architect_returns_taskplan_with_expected_steps() {
    let summary = "Test summary".to_string();
    let secondary = vec![Archetype::Oracle, Archetype::Empath];

    let plan = plan_task(summary, secondary.clone()).await;

    assert_eq!(plan.steps.len(), 1 + secondary.len());
    assert_eq!(plan.steps[0].archetype, Archetype::Architect);
}
