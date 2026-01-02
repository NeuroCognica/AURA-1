use crate::execution::types::*;
use super::{AdapterError, ArchetypeAdapter};
use async_trait::async_trait;

pub struct DeterministicAdapter {
    id: ArchetypeId,
}

impl DeterministicAdapter {
    pub fn new(id: ArchetypeId) -> Self { Self { id } }
}

#[async_trait]
impl ArchetypeAdapter for DeterministicAdapter {
    fn archetype(&self) -> ArchetypeId { self.id }

    async fn run(&self, input: AdapterInput) -> Result<AdapterOutput, AdapterError> {
        if input.request_id.trim().is_empty() {
            return Err(AdapterError::InvalidInput("request_id empty".into()));
        }

        Ok(AdapterOutput {
            request_id: input.request_id,
            archetype: self.id,
            proposals: vec![],
            explanation: format!(
                "Deterministic {:?} adapter active. No proposals in Phase 4 Step 1.",
                self.id
            ),
        })
    }
}
