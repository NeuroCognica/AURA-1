use std::sync::Arc;
use std::collections::BTreeMap;

use orchestrator::llm::{LLMClient, LLMResponse, LLMRequest};
use orchestrator::execution::adapters::technician::TechnicianLlmAdapter;
use orchestrator::execution::types::{AdapterInput, AdapterOutput, ArchetypeId, ProposedAction, RiskLevel};
use orchestrator::execution::adapters::AdapterError;
use orchestrator::execution::adapters::ArchetypeAdapter;

struct FakeLlm {
    resp: String,
}

#[async_trait::async_trait]
impl LLMClient for FakeLlm {
    async fn generate(&self, _req: LLMRequest) -> Result<LLMResponse, String> {
        Ok(LLMResponse { text: self.resp.clone() })
    }
}

#[tokio::test]
async fn technician_parses_valid_json_response() {
    let out = AdapterOutput {
        request_id: "r-1".into(),
        archetype: ArchetypeId::Technician,
        proposals: vec![ProposedAction {
            action_type: "notes.create".into(),
            payload: serde_json::json!({"title":"x"}),
            risk: RiskLevel::Low,
            rationale: "reason".into(),
        }],
        explanation: "ok".into(),
    };

    let resp_text = serde_json::to_string(&out).unwrap();

    let llm = Arc::new(FakeLlm { resp: resp_text });
    let adapter = TechnicianLlmAdapter::new(llm, "model-a".into(), 0.2, 0.9);

    let input = AdapterInput {
        request_id: "r-1".into(),
        user_text: "do it".into(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let res = adapter.run(input).await.expect("should parse");
    assert_eq!(res.proposals.len(), 1);
}

#[tokio::test]
async fn technician_fails_on_invalid_json() {
    let llm = Arc::new(FakeLlm { resp: "not json".into() });
    let adapter = TechnicianLlmAdapter::new(llm, "model-a".into(), 0.2, 0.9);

    let input = AdapterInput {
        request_id: "r-2".into(),
        user_text: "do it".into(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let res = adapter.run(input).await;
    match res {
        Err(AdapterError::InvalidOutput(_)) => {}
        other => panic!("expected InvalidOutput, got {:?}", other),
    }
}
