use crate::council::CouncilMsg;
use crate::llm::{LLMClient, LLMRequest};
use crate::prompts::build_sentinel_prompt;
use crate::response_handler::process_llm_response;
use crate::authority_client::broadcast_response;

/// Handle a single CouncilMsg. This implements the linear pipeline:
/// Query -> prompt -> LLM -> validation -> broadcast
pub async fn handle_query(msg: CouncilMsg, llm: &dyn LLMClient) -> Result<(), String> {
    match msg {
        CouncilMsg::Query { .. } => {
            // Build prompt (string only)
            let prompt = build_sentinel_prompt(&msg);
            println!("[live] built prompt: {}", prompt);

            // Call LLM
            let req = LLMRequest { prompt };
            let llm_res = llm.generate(req).await.map_err(|e| format!("llm error: {}", e))?;
            println!("[live] llm raw output: {}", llm_res.raw);

            // Process/validate response
            let processed = process_llm_response(&llm_res.raw, "Sentinel")
                .map_err(|e| format!("response validation error: {}", e))?;
            println!("[live] processed response: {:?}", processed);

            // Broadcast via authority client
            broadcast_response(processed).await.map_err(|e| format!("broadcast error: {}", e))?;
            println!("[live] broadcasted response");

            Ok(())
        }
        other => {
            println!("[live] ignoring non-query message: {:?}", other);
            Ok(())
        }
    }
}
