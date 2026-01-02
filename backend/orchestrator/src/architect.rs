use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::intent_classifier::Archetype;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubTask {
    pub archetype: Archetype,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskPlan {
    pub steps: Vec<SubTask>,
}

/// Deterministic, synchronous planner stub.
/// - One primary step: `Architect` with the task summary
/// - One subtask per `secondary` archetype
pub async fn plan_task(summary: String, secondary: Vec<Archetype>) -> TaskPlan {
    let mut steps: Vec<SubTask> = Vec::new();

    // Primary Architect step
    let architect_input = json!({ "task_summary": summary });
    steps.push(SubTask {
        archetype: Archetype::Architect,
        input: architect_input,
    });

    // One subtask per secondary archetype
    for arch in secondary.iter() {
        let input = json!({ "task_summary": "", "assigned_by": "architect" });
        steps.push(SubTask {
            archetype: arch.clone(),
            input,
        });
    }

    TaskPlan { steps }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_classifier::Archetype;

    #[tokio::test]
    async fn produces_taskplan_with_secondary_steps() {
        let summary = "Do a thing".to_string();
        let secondary = vec![Archetype::Oracle, Archetype::Jester];

        let plan = plan_task(summary.clone(), secondary.clone()).await;

        // one primary Architect + two secondary
        assert_eq!(plan.steps.len(), 1 + secondary.len());
        assert_eq!(plan.steps[0].archetype, Archetype::Architect);
    }

    #[tokio::test]
    async fn deterministic_and_no_mutation() {
        let summary = "Build me a plan".to_string();
        let secondary = vec![Archetype::Explorer];
        let secondary_clone = secondary.clone();

        let a = plan_task(summary.clone(), secondary.clone()).await;
        let b = plan_task(summary.clone(), secondary.clone()).await;

        // deterministic
        assert_eq!(a, b);

        // no mutation of the input vector
        assert_eq!(secondary, secondary_clone);
    }
}
