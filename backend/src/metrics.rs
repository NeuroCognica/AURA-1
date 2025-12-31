// Lightweight metrics scaffold guarded by the `metrics` feature.
// This file provides a minimal in-process counter for replay/persistence metrics
// without introducing external dependencies. Enable with `--features metrics`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Public metrics handle. When the `metrics` feature is disabled, this is a no-op
/// stub with the same API so the rest of the crate can call it safely.
#[cfg(feature = "metrics")]
#[derive(Clone)]
pub struct Metrics {
    pub replay_last_seq: Arc<AtomicU64>,
}

#[cfg(feature = "metrics")]
impl Metrics {
    pub fn new() -> Self {
        Self {
            replay_last_seq: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_last_seq(&self, v: u64) {
        self.replay_last_seq.store(v, Ordering::Relaxed);
    }

    pub fn get_last_seq(&self) -> u64 {
        self.replay_last_seq.load(Ordering::Relaxed)
    }
}

// No-op stub when metrics feature is not enabled.
#[cfg(not(feature = "metrics"))]
#[derive(Clone)]
pub struct Metrics {}

#[cfg(not(feature = "metrics"))]
impl Metrics {
    pub fn new() -> Self {
        Self {}
    }
    pub fn set_last_seq(&self, _v: u64) {}
    pub fn get_last_seq(&self) -> u64 {
        0
    }
}
