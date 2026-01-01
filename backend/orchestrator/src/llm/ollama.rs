use super::{LLMClient, LLMRequest, LLMResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaClient {
    http: Client,
    endpoint: String,
}

impl OllamaClient {
    pub fn new(endpoint: String) -> Self {
        Self { http: Client::new(), endpoint }
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
    async fn generate(&self, req: LLMRequest) -> Result<LLMResponse, String> {
        let body = OllamaGenerateReq {
            model: &req.model,
            prompt: &req.prompt,
            temperature: req.temperature,
            top_p: req.top_p,
            stream: false,
        };

        let url = format!("{}/api/generate", self.endpoint.trim_end_matches('/'));
        let resp = self.http.post(url).json(&body).send().await
            .map_err(|e| format!("ollama transport: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("ollama status: {}", resp.status()));
        }

        let parsed: OllamaGenerateResp = resp.json().await
            .map_err(|e| format!("ollama parse: {}", e))?;

        Ok(LLMResponse { text: parsed.response })
    }
}
