use super::{LLMClient, LLMError, LLMRequest, LLMResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

pub struct OllamaClient {
    client: Client,
    endpoint: String,
    model: String,
    temperature: f32,
    top_p: f32,
}

impl OllamaClient {
    pub fn new(endpoint: String, model: String, temperature: f32, top_p: f32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            endpoint,
            model,
            temperature,
            top_p,
        }
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMError> {
        let body = json!({
            "model": self.model,
            "prompt": request.prompt,
            "stream": false,
            "options": {
                "temperature": self.temperature,
                "top_p": self.top_p
            }
        });

        let url = format!("{}/api/generate", self.endpoint);

        let mut attempts = 0u8;
        loop {
            attempts = attempts.saturating_add(1);

            let res = self.client.post(&url).json(&body).send().await;

            match res {
                Ok(resp) => {
                    let value: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|_| LLMError::InvalidResponse)?;

                    let text = value
                        .get("response")
                        .and_then(|v| v.as_str())
                        .ok_or(LLMError::InvalidResponse)?;

                    return Ok(LLMResponse { raw: text.to_string() });
                }
                Err(e) if attempts < 3 => {
                    tokio::time::sleep(Duration::from_millis(250u64 * attempts as u64)).await;
                    continue;
                }
                Err(e) => {
                    return Err(LLMError::Transport(e.to_string()));
                }
            }
        }
    }
}
