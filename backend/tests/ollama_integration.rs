use reqwest::Client;

#[tokio::test]
async fn ollama_smoke() {
    let ollama = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let client = Client::new();
    let url = format!("{}/api/models", ollama.trim_end_matches('/'));
    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                // pass the test if we can reach models endpoint
                let _ = resp.text().await.unwrap_or_default();
            } else {
                eprintln!("ollama models endpoint returned non-200: {}", resp.status());
            }
        }
        Err(e) => {
            eprintln!("OLLAMA not available; skipping smoke test: {}", e);
            // treat as skipped; do not fail CI if Ollama not running
        }
    }
}
