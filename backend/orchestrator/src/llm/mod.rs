use async_trait::async_trait;

/// LLM generation parameters
pub struct GenerationParams {
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
}

/// LLM response
#[derive(Debug)]
pub struct LLMResponse {
    pub text: String,
}

/// LLM client trait (transport layer only)
/// 
/// CRITICAL: Client does not:
/// - Load prompts from files
/// - Know about archetypes
/// - Verify provenance
/// - Make policy decisions
/// 
/// Client responsibility: transmit system_prompt + user_prompt to model, return text.
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Generate LLM response with mandatory system prompt.
    /// 
    /// # Arguments
    /// * `system_prompt` - Constitutional prompt (MUST be non-empty)
    /// * `user_prompt` - User context (may be empty)
    /// * `params` - Model parameters
    /// 
    /// # Errors
    /// Returns error if system_prompt is empty/whitespace or transport fails.
    async fn generate(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        params: &GenerationParams,
    ) -> Result<LLMResponse, String>;
}

pub mod ollama;

pub use ollama::OllamaClient;
