use orchestrator::intent_classifier::{Archetype, IntentClassification, ClassifierError};
use orchestrator::router::{route_intent, route_intent_from_result, RoutingDecision};

#[test]
fn authority_query_routes_to_sentinel() {
    let intent = IntentClassification {
        primary_archetype: Archetype::Explorer,
        secondary_archetypes: vec![],
        task_summary: "Check constitution".to_string(),
        is_authority_query: true,
    };

    let d = route_intent(intent);
    assert!(matches!(d, RoutingDecision::ToSentinel));
}

#[test]
fn architect_primary_routes_to_architect() {
    let intent = IntentClassification {
        primary_archetype: Archetype::Architect,
        secondary_archetypes: vec![Archetype::Oracle, Archetype::Explorer],
        task_summary: "Plan migration".to_string(),
        is_authority_query: false,
    };

    let d = route_intent(intent);
    match d {
        RoutingDecision::ToArchitect { secondary, task_summary } => {
            assert_eq!(secondary.len(), 2);
            assert_eq!(task_summary, "Plan migration");
        }
        _ => panic!("expected ToArchitect"),
    }
}

#[test]
fn non_architect_primary_routes_to_single() {
    let intent = IntentClassification {
        primary_archetype: Archetype::Oracle,
        secondary_archetypes: vec![],
        task_summary: "Assess risk".to_string(),
        is_authority_query: false,
    };

    let d = route_intent(intent);
    match d {
        RoutingDecision::ToSingleArchetype { archetype, task_summary } => {
            assert_eq!(archetype, Archetype::Oracle);
            assert_eq!(task_summary, "Assess risk");
        }
        _ => panic!("expected ToSingleArchetype"),
    }
}

#[test]
fn classifier_error_routes_to_sentinel() {
    let d = route_intent_from_result(Err(ClassifierError::InvalidSchema));
    assert!(matches!(d, RoutingDecision::ToSentinel));
}
