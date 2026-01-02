use std::collections::HashMap;
use std::sync::Arc;

use crate::config::OrchestratorConfig;
use crate::execution::adapters::{ArchetypeAdapter, deterministic};
use crate::execution::adapters::technician::TechnicianLlmAdapter;
use crate::execution::types::ArchetypeId;
use crate::llm::LLMClient;
use crate::intent_classifier::Archetype as IcArchetype;

pub struct AdapterRegistry {
    adapters: HashMap<ArchetypeId, Box<dyn ArchetypeAdapter>>,
}

impl AdapterRegistry {
    /// Build registry from validated config and an injected LLM client.
    /// The provided `llm` is used for any LLM-backed adapters (Technician).
    /// Missing entries are considered startup failure.
    pub fn from_config_with_llm(cfg: &OrchestratorConfig, llm: Arc<dyn LLMClient>) -> Result<Self, String> {
        let mut adapters: HashMap<ArchetypeId, Box<dyn ArchetypeAdapter>> = HashMap::new();

        // Create deterministic adapters for canonical archetypes
        adapters.insert(ArchetypeId::Architect, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Architect)));
        adapters.insert(ArchetypeId::Explorer, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Explorer)));
        adapters.insert(ArchetypeId::Oracle, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Oracle)));
        adapters.insert(ArchetypeId::Mentor, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Mentor)));
        adapters.insert(ArchetypeId::Empath, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Empath)));
        adapters.insert(ArchetypeId::Jester, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Jester)));
        adapters.insert(ArchetypeId::Sentinel, Box::new(deterministic::DeterministicAdapter::new(ArchetypeId::Sentinel)));

        // Instantiate Technician LLM-backed adapter using model/params from a validated archetype config.
        // We pick the `Architect` archetype parameters as the canonical Technician model when not otherwise specified.
        let arch_cfg = cfg.model_for(IcArchetype::Architect);
        let tech = TechnicianLlmAdapter::new(
            llm,
            arch_cfg.model.clone(),
            arch_cfg.temperature as f32,
            arch_cfg.top_p as f32,
        );
        adapters.insert(ArchetypeId::Technician, Box::new(tech));

        Ok(AdapterRegistry { adapters })
    }

    /// Convenience: build from config file path by constructing a default Ollama client.
    pub fn from_config_file(path: &str) -> Result<Self, String> {
        let cfg = OrchestratorConfig::load_from_file(path).map_err(|e| format!("config load: {}", e))?;
        let endpoint = cfg.llm_endpoint().to_string();
        let client = Arc::new(crate::llm::OllamaClient::new(endpoint));
        Self::from_config_with_llm(&cfg, client)
    }

    pub fn get(&self, id: &ArchetypeId) -> Option<&dyn ArchetypeAdapter> {
        self.adapters.get(id).map(|b| &**b as &dyn ArchetypeAdapter)
    }
}
