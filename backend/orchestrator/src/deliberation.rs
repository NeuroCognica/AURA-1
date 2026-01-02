use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

use crate::intent_classifier::Archetype;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: Uuid,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeRationale {
    pub archetype: Archetype,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliberation {
    pub id: Uuid,
    pub task_id: Uuid,
    pub raised_by: Archetype,
    pub summary: String,
    pub options: Vec<DecisionOption>,
    pub recommended: Option<Uuid>,
    pub supporting: Vec<ArchetypeRationale>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    pub choice_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub enum DeliberationError {
    Canceled,
    Closed,
}

/// Manager that tracks outstanding deliberations and provides wait/submit semantics.
#[derive(Clone, Default)]
pub struct DeliberationManager {
    inner: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Decision>>>>,
    canceled: Arc<Mutex<HashSet<Uuid>>>,
}

impl DeliberationManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            canceled: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Create a deliberation and a receiver the orchestrator can await.
    pub async fn create_deliberation(
        &self,
        task_id: Uuid,
        raised_by: Archetype,
        summary: String,
        options: Vec<DecisionOption>,
        recommended: Option<Uuid>,
        supporting: Vec<ArchetypeRationale>,
    ) -> (Deliberation, DeliberationHandle) {
        let id = Uuid::new_v4();
        let d = Deliberation {
            id,
            task_id,
            raised_by,
            summary,
            options,
            recommended,
            supporting,
        };

        let (tx, rx) = oneshot::channel::<Decision>();
        self.inner.lock().await.insert(id, tx);

        (d, DeliberationHandle { id, recv: Some(rx), manager: self.clone() })
    }

    /// Submit a decision for an existing deliberation. Returns true if delivered.
    pub async fn submit_decision(&self, id: Uuid, decision: Decision) -> bool {
        if let Some(tx) = self.inner.lock().await.remove(&id) {
            let _ = tx.send(decision);
            true
        } else {
            false
        }
    }

    /// Mark a deliberation canceled and remove the sender.
    pub async fn cancel_deliberation(&self, id: Uuid) {
        self.canceled.lock().await.insert(id);
        let _ = self.inner.lock().await.remove(&id);
    }
}

/// Handle returned from `create_deliberation` that can be awaited.
pub struct DeliberationHandle {
    pub id: Uuid,
    recv: Option<oneshot::Receiver<Decision>>,
    manager: DeliberationManager,
}

impl DeliberationHandle {
    /// Await the human decision. This represents a pause point in coordination.
    /// Returns Ok(Decision) when a human submits one, or Err when canceled/closed.
    pub async fn wait_for_decision(mut self) -> Result<Decision, DeliberationError> {
        if let Some(rx) = self.recv.take() {
            match rx.await {
                Ok(dec) => Ok(dec),
                Err(_) => {
                    // determine if explicitly canceled
                    let canceled = self.manager.canceled.lock().await.contains(&self.id);
                    if canceled {
                        Err(DeliberationError::Canceled)
                    } else {
                        Err(DeliberationError::Closed)
                    }
                }
            }
        } else {
            Err(DeliberationError::Closed)
        }
    }

    /// Cancel the deliberation without a decision (cleanup).
    pub async fn cancel(self) {
        self.manager.cancel_deliberation(self.id).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_classifier::Archetype;

    #[tokio::test]
    async fn create_and_wait_and_submit_decision() {
        let manager = DeliberationManager::new();

        let option = DecisionOption { id: Uuid::new_v4(), label: "Proceed".to_string() };

        let (d, handle) = manager
            .create_deliberation(
                Uuid::new_v4(),
                Archetype::Architect,
                "Conflict between Oracle and Jester".to_string(),
                vec![option.clone()],
                Some(option.id),
                vec![],
            )
            .await;

        let id = d.id;

        // spawn a waiter
        let waiter = tokio::spawn(async move { handle.wait_for_decision().await });

        // submit a decision
        let ok = manager
            .submit_decision(
                id,
                Decision { choice_id: option.id, notes: None },
            )
            .await;

        assert!(ok, "decision should be delivered");

        let result = waiter.await.unwrap();
        assert_eq!(result.unwrap().choice_id, option.id);
    }

    #[tokio::test]
    async fn submit_nonexistent_deliberation_returns_false() {
        let manager = DeliberationManager::new();
        let fake_id = Uuid::new_v4();
        let ok = manager
            .submit_decision(
                fake_id,
                Decision { choice_id: Uuid::new_v4(), notes: None },
            )
            .await;
        assert!(!ok);
    }
}
