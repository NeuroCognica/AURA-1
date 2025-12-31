use serde_json::Value;
use crate::council::CouncilMsg;

#[derive(thiserror::Error, Debug)]
pub enum ResponseError {
    #[error("invalid json")]
    InvalidJson,
    #[error("forbidden content")]
    Forbidden,
}

pub fn process_llm_response(raw: &str, archetype: &str) -> Result<CouncilMsg, ResponseError> {
    let v: Value = serde_json::from_str(raw).map_err(|_| ResponseError::InvalidJson)?;

    let analysis = v
        .get("analysis")
        .and_then(|v| v.as_str())
        .ok_or(ResponseError::InvalidJson)?;

    let lowered = analysis.to_lowercase();
    let forbidden = ["must", "should", "execute", "allow", "deny", "override"];

    if forbidden.iter().any(|w| lowered.contains(w)) {
        return Err(ResponseError::Forbidden);
    }

    Ok(CouncilMsg::Response { archetype: archetype.to_string(), analysis: analysis.to_string() })
}
