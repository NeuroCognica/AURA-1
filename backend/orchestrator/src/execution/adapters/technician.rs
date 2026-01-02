use crate::execution::adapters::{AdapterError, ArchetypeAdapter};
use crate::execution::types::{AdapterInput, AdapterOutput, ArchetypeId};
use crate::llm::{GenerationParams, LLMClient};
use async_trait::async_trait;
use std::sync::Arc;

pub struct TechnicianLlmAdapter {
    llm: Arc<dyn LLMClient>,
    model: String,
    temperature: f32,
    top_p: f32,
    id: ArchetypeId,
}

impl TechnicianLlmAdapter {
    pub fn new(llm: Arc<dyn LLMClient>, model: String, temperature: f32, top_p: f32) -> Self {
        Self {
            llm,
            model,
            temperature,
            top_p,
            id: ArchetypeId::Technician,
        }
    }
}

#[async_trait]
impl ArchetypeAdapter for TechnicianLlmAdapter {
    fn archetype(&self) -> ArchetypeId {
        self.id
    }

    async fn run(&self, input: AdapterInput) -> Result<AdapterOutput, AdapterError> {
        if input.request_id.trim().is_empty() {
            return Err(AdapterError::InvalidInput("request_id empty".into()));
        }

        // System prompt: instructs LLM to output JSON matching AdapterOutput only
        let system_prompt = "Respond with JSON only matching the AdapterOutput schema. No explanations, no extra text.";

        // User prompt: the actual request data
        let user_prompt = format!(
            "RequestId: {}\nUserText: {}\nContext: {:?}",
            input.request_id, input.user_text, input.context
        );

        let params = GenerationParams {
            model: self.model.clone(),
            temperature: self.temperature,
            top_p: self.top_p,
        };

        let resp = self
            .llm
            .generate(system_prompt, &user_prompt, &params)
            .await
            .map_err(|e| AdapterError::Transport(e))?;

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
