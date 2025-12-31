use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct LLMRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LLMResponse {
    pub raw: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("transport error: {0}")]
    Transport(String),

    #[error("timeout")]
    Timeout,

    #[error("invalid response")]
    InvalidResponse,
}

#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError>;
}
