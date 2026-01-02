use thiserror::Error;
use async_trait::async_trait;

use crate::execution::types::{AdapterInput, AdapterOutput};
use crate::execution::types::ArchetypeId;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("model output invalid: {0}")]
    InvalidOutput(String),
    #[error("transport error: {0}")]
    Transport(String),
}

#[async_trait]
pub trait ArchetypeAdapter: Send + Sync {
    fn archetype(&self) -> ArchetypeId;
    async fn run(&self, input: AdapterInput) -> Result<AdapterOutput, AdapterError>;
}

pub mod deterministic;
pub mod technician;
