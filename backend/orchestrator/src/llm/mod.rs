use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LLMRequest {
    pub model: String,
    pub prompt: String,
    pub temperature: f32,
    pub top_p: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LLMResponse {
    pub text: String,
}

#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate(&self, req: LLMRequest) -> Result<LLMResponse, String>;
}

pub mod ollama;

pub use ollama::OllamaClient;
