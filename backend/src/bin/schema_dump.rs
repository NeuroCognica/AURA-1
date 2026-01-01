use aura_backend::council_verdict::*;
use schemars::schema_for;

/// Output format: one JSON object per line with {"struct": "Name", "schema": {...}}
fn main() {
    // CouncilVerdict
    let verdict_schema = schema_for!(CouncilVerdict);
    println!(
        "{}",
        serde_json::json!({
            "struct": "CouncilVerdict",
            "schema": verdict_schema
        })
    );

    // CouncilEnvelope
    let envelope_schema = schema_for!(CouncilEnvelope);
    println!(
        "{}",
        serde_json::json!({
            "struct": "CouncilEnvelope",
            "schema": envelope_schema
        })
    );

    // CouncilMsg
    let msg_schema = schema_for!(CouncilMsg);
    println!(
        "{}",
        serde_json::json!({
            "struct": "CouncilMsg",
            "schema": msg_schema
        })
    );
}
