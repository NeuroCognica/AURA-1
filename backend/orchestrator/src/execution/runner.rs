use crate::verdict_loop::{AuthorityClient, request_verdict, Verdict, VerdictError};
use crate::verdict_loop::ProposedAction as VerdictProposedAction;

#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    DryRun,
    Live,
}

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("sentinel denied: {0}")]
    Denied(String),
    #[error("sentinel refused: {0}")]
    Refused(String),
    #[error("sentinel error: {0}")]
    SentinelError(String),
    #[error("tool error: {0}")]
    ToolError(String),
}

/// The single canonical path from proposal -> execution.
/// This function MUST synchronously await a Sentinel verdict and
/// enforce that DryRun is safe (no side effects). Live execution
/// is intentionally not implemented and returns an error.
pub async fn run_proposal(
    mode: ExecutionMode,
    authority: &dyn AuthorityClient,
    proposal: VerdictProposedAction,
) -> Result<(), ExecutionError> {
    let v = request_verdict(proposal, authority).await
        .map_err(|e| ExecutionError::SentinelError(e.to_string()))?;

    match v {
        Verdict::Approve => match mode {
            ExecutionMode::DryRun => {
                // Log-only path; no side effects.
                Ok(())
            }
            ExecutionMode::Live => Err(ExecutionError::ToolError(
                "Live execution not enabled yet. Use DryRun.".into()
            )),
        },
        Verdict::Deny => Err(ExecutionError::Denied("Denied by Sentinel".into())),
        Verdict::Refuse => Err(ExecutionError::Refused("Refused by Sentinel".into())),
    }
}
