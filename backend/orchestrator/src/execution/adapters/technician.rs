use std::sync::Arc;
use async_trait::async_trait;

use crate::execution::types::{AdapterInput, AdapterOutput, ArchetypeId};
use crate::execution::adapters::{AdapterError, ArchetypeAdapter};
use crate::llm::{LLMClient, LLMRequest};

pub struct TechnicianLlmAdapter {
    llm: Arc<dyn LLMClient>,
    model: String,
    temperature: f32,
    top_p: f32,
    id: ArchetypeId,
}

impl TechnicianLlmAdapter {
    pub fn new(llm: Arc<dyn LLMClient>, model: String, temperature: f32, top_p: f32) -> Self {
        Self { llm, model, temperature, top_p, id: ArchetypeId::Technician }
    }
}

#[async_trait]
impl ArchetypeAdapter for TechnicianLlmAdapter {
    fn archetype(&self) -> ArchetypeId { self.id }

    async fn run(&self, input: AdapterInput) -> Result<AdapterOutput, AdapterError> {
        if input.request_id.trim().is_empty() {
            return Err(AdapterError::InvalidInput("request_id empty".into()));
        }

        // Build a strict prompt that instructs the LLM to output JSON matching AdapterOutput only.
        let prompt = format!(
            "Respond with JSON only matching the AdapterOutput schema.\nRequestId: {}\nUserText: {}\nContext: {:?}\n",
            input.request_id, input.user_text, input.context
        );

        let req = LLMRequest {
            model: self.model.clone(),
            prompt,
            temperature: self.temperature,
            top_p: self.top_p,
        };

        let resp = self.llm.generate(req).await.map_err(|e| AdapterError::Transport(e))?;

        // MUST parse strict JSON matching AdapterOutput; fail-closed on any deviation.
        let parsed: AdapterOutput = serde_json::from_str(&resp.text)
            .map_err(|e| AdapterError::InvalidOutput(format!("json parse error: {}", e)))?;

        // Ensure request_id matches
        if parsed.request_id != input.request_id {
            return Err(AdapterError::InvalidOutput("mismatched request_id".into()));
        }

        Ok(parsed)
    }
}
