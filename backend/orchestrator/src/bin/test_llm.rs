// Test binary for live Ollama conversation with constitutional compliance
use orchestrator::config::OrchestratorConfig;
use orchestrator::llm::ollama::OllamaClient;
use orchestrator::llm::GenerationParams;
use orchestrator::prompts::{ArchetypeId, PromptLoader};
use orchestrator::constitutional::ConstitutionalInvoker;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== AURA Constitutional LLM Test ===\n");

    // Load config
    let config = OrchestratorConfig::load_from_file("backend/orchestrator/orchestrator.json")?;
    println!("✓ Loaded orchestrator config");
    
    // Initialize OllamaClient
    let ollama_endpoint = config.llm_endpoint().to_string();
    let ollama_client = OllamaClient::new(ollama_endpoint);
    println!("✓ Initialized Ollama client");

    // Initialize PromptLoader
    let prompt_dir = std::path::PathBuf::from("backend/orchestrator/prompts");
    println!("✓ Initialized prompt directory");

    // Initialize ConstitutionalInvoker
    let invoker = ConstitutionalInvoker::new(prompt_dir)?;
    println!("✓ Initialized constitutional invoker");

    println!("\n=== Testing Sentinel (temperature=0.0, deterministic) ===\n");

    // Get Sentinel config
    let sentinel_config = config.archetypes.get("Sentinel")
        .expect("Sentinel config must exist");

    let params = GenerationParams {
        model: sentinel_config.model.clone(),
        temperature: sentinel_config.temperature as f32,
        top_p: sentinel_config.top_p as f32,
    };

    println!("Model: {}", params.model);
    println!("Temperature: {}", params.temperature);
    println!("Top-p: {}\n", params.top_p);

    // Interactive loop
    loop {
        print!("Enter user message (or 'quit' to exit): ");
        io::stdout().flush()?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.is_empty() {
            continue;
        }

        if user_input == "quit" {
            break;
        }

        println!("\n--- Invoking Sentinel through constitutional layer ---");

        match invoker.generate(
            ArchetypeId::Sentinel,
            None,  // No username for test
            user_input,
            &params,
            &ollama_client
        ).await {
            Ok(response) => {
                println!("\n✓ Sentinel Response:");
                println!("{}\n", response.text);

                // TODO: Parse DECISION/ARTICLE/REASON format
                // TODO: Log Forever Law compliance (article citations)
                // TODO: Generate QSIC provenance hash
            }
            Err(e) => {
                println!("\n✗ Error: {:?}\n", e);
            }
        }
    }

    println!("\nGoodbye!");
    Ok(())
}
