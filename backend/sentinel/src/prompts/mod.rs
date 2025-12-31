use std::fs;
use crate::council::CouncilMsg;

pub fn build_sentinel_prompt(msg: &CouncilMsg) -> String {
    let system = fs::read_to_string("src/prompts/sentinel_system.txt")
        .expect("sentinel system prompt missing");

    let context = format!(
        "Council Query:\n{}\n",
        serde_json::to_string_pretty(msg).unwrap()
    );

    format!("{}\n\n{}", system, context)
}
