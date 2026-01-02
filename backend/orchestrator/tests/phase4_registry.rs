use std::sync::Arc;
use std::collections::BTreeMap;

use orchestrator::execution::registry::AdapterRegistry;
use orchestrator::execution::types::AdapterInput;
use orchestrator::execution::types::ArchetypeId;
use orchestrator::llm::LLMClient;
use orchestrator::llm::{LLMResponse, GenerationParams};

struct FakeLlm { resp: String }

#[async_trait::async_trait]
impl LLMClient for FakeLlm {
    async fn generate(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _params: &GenerationParams,
    ) -> Result<LLMResponse, String> {
        Ok(LLMResponse { text: self.resp.clone() })
    }
}

#[tokio::test]
async fn technician_adapter_registered_and_used() {
    // Load config from manifest and inject fake llm that returns a valid AdapterOutput JSON
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let cfg = orchestrator::config::OrchestratorConfig::load_from_file(&path).expect("config load");

    // Build a fake AdapterOutput JSON that the Technician adapter will parse
    let out = orchestrator::execution::types::AdapterOutput {
        request_id: "r-tech".into(),
        archetype: ArchetypeId::Technician,
        proposals: vec![orchestrator::execution::types::ProposedAction {
            action_type: "notes.create".into(),
            payload: serde_json::json!({"title":"t"}),
            risk: orchestrator::execution::types::RiskLevel::Low,
            rationale: "ok".into(),
        }],
        explanation: "ok".into(),
    };

    let fake_llm = Arc::new(FakeLlm { resp: serde_json::to_string(&out).unwrap() });
    let reg = AdapterRegistry::from_config_with_llm(&cfg, fake_llm).expect("registry");

    let adapter = reg.get(&ArchetypeId::Technician).expect("technician present");

    let input = AdapterInput {
        request_id: "r-tech".into(),
        user_text: "do".into(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let out = adapter.run(input).await.expect("adapter run");
    assert_eq!(out.proposals.len(), 1);
}

#[tokio::test]
async fn deterministic_adapters_still_work_and_dryrun_path() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let cfg = orchestrator::config::OrchestratorConfig::load_from_file(&path).expect("config load");

    // inject a fake llm returning empty output; deterministic adapters used for most archetypes
    let fake_llm = Arc::new(FakeLlm { resp: "{}".into() });
    let reg = AdapterRegistry::from_config_with_llm(&cfg, fake_llm).expect("registry");

    let adapter = reg.get(&ArchetypeId::Architect).expect("architect adapter");
    let input = AdapterInput {
        request_id: "r-arch".into(),
        user_text: "plan".into(),
        context: BTreeMap::new(),
        archetype: orchestrator::intent_classifier::Archetype::Architect,
    };

    let out = adapter.run(input).await.expect("deterministic run");
    // deterministic adapter returns no proposals in Phase 4 Step 1
    assert_eq!(out.proposals.len(), 0);
}
