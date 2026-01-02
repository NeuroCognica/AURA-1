use crate::intent_classifier::{Archetype, IntentClassification};

#[derive(Debug, Clone, PartialEq)]
pub enum RoutingDecision {
    ToSentinel,
    ToArchitect {
        secondary: Vec<Archetype>,
        task_summary: String,
    },
    ToSingleArchetype {
        archetype: Archetype,
        task_summary: String,
    },
}

/// Route an already-parsed intent into a routing decision.
pub fn route_intent(intent: IntentClassification) -> RoutingDecision {
    if intent.is_authority_query {
        return RoutingDecision::ToSentinel;
    }

    match intent.primary_archetype {
        Archetype::Architect => RoutingDecision::ToArchitect {
            secondary: intent.secondary_archetypes,
            task_summary: intent.task_summary,
        },
        other => RoutingDecision::ToSingleArchetype {
            archetype: other,
            task_summary: intent.task_summary,
        },
    }
}

/// Helper that accepts a classifier result and fails-closed to Sentinel on error.
pub fn route_intent_from_result(
    intent_res: Result<IntentClassification, crate::intent_classifier::ClassifierError>,
) -> RoutingDecision {
    match intent_res {
        Ok(intent) => route_intent(intent),
        Err(_) => RoutingDecision::ToSentinel,
    }
}
