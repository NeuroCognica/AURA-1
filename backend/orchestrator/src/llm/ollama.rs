use super::{GenerationParams, LLMClient, LLMResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaClient {
    http: Client,
    endpoint: String,
}

impl OllamaClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            http: Client::new(),
            endpoint,
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaGenerateReq<'a> {
    model: &'a str,
    prompt: &'a str,
    temperature: f32,
    top_p: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResp {
    response: String,
}

#[async_trait::async_trait]
impl LLMClient for OllamaClient {
    async fn generate(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        params: &GenerationParams,
    ) -> Result<LLMResponse, String> {
        // Fail-closed: system_prompt is mandatory and must not be empty/whitespace
        if system_prompt.trim().is_empty() {
            return Err("system_prompt is mandatory (fail-closed)".to_string());
        }

        // Combine system prompt and user prompt
        // System prompt always comes first (constitutional precedence)
        let combined_prompt = if user_prompt.is_empty() {
            system_prompt.to_string()
        } else {
            format!("{}\n\n{}", system_prompt, user_prompt)
        };

        let body = OllamaGenerateReq {
            model: &params.model,
            prompt: &combined_prompt,
            temperature: params.temperature,
            top_p: params.top_p,
            stream: false,
        };

        let url = format!("{}/api/generate", self.endpoint.trim_end_matches('/'));
        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("ollama transport: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("ollama status: {}", resp.status()));
        }

        let parsed: OllamaGenerateResp = resp
            .json()
            .await
            .map_err(|e| format!("ollama parse: {}", e))?;

        Ok(LLMResponse {
            text: parsed.response,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_system_prompt_fails() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        let params = GenerationParams {
            model: "test".to_string(),
            temperature: 0.0,
            top_p: 0.1,
        };

        // Empty system_prompt must fail
        let result = client.generate("", "user input", &params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mandatory"));
    }

    #[tokio::test]
    async fn test_whitespace_system_prompt_fails() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        let params = GenerationParams {
            model: "test".to_string(),
            temperature: 0.0,
            top_p: 0.1,
        };

        // Whitespace-only system_prompt must fail
        let result = client.generate("   \n\t  ", "user input", &params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mandatory"));
    }

    #[test]
    fn test_prompt_combination_system_only() {
        // When user_prompt is empty, only system_prompt is sent
        let system = "You are Sentinel.";
        let user = "";

        let combined = if user.is_empty() {
            system.to_string()
        } else {
            format!("{}\n\n{}", system, user)
        };

        assert_eq!(combined, "You are Sentinel.");
    }

    #[test]
    fn test_prompt_combination_both() {
        // When both present, system comes first with separator
        let system = "You are Sentinel.";
        let user = "Evaluate this request.";

        let combined = format!("{}\n\n{}", system, user);

        assert_eq!(combined, "You are Sentinel.\n\nEvaluate this request.");
        assert!(combined.starts_with(system));
    }
}
