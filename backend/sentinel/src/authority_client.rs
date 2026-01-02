use crate::council::CouncilMsg;
use std::sync::{Mutex, OnceLock};

/// Persist-first broadcast skeleton. Intentionally minimal.
pub async fn broadcast_response(msg: CouncilMsg) -> Result<(), String> {
    // If a test hook is registered, call it and return its result.
    if let Some(m) = TEST_HOOK.get() {
        if let Ok(guard) = m.lock() {
            if let Some(h) = guard.as_ref() {
                return h(msg);
            }
        }
    }

    // TODO: implement HTTP POST to AURA-1 `/api/council/broadcast`
    // Wait for ACK and retry with backoff on failure.
    Ok(())
}

// Test-only hook to observe broadcasts without contacting AURA-1.
static TEST_HOOK: OnceLock<Mutex<Option<Box<dyn Fn(CouncilMsg) -> Result<(), String> + Send + Sync>>>> = OnceLock::new();

pub fn set_broadcast_hook(hook: Box<dyn Fn(CouncilMsg) -> Result<(), String> + Send + Sync>) {
    let m = TEST_HOOK.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = m.lock() {
        *guard = Some(hook);
    }
}

pub fn clear_broadcast_hook() {
    if let Some(m) = TEST_HOOK.get() {
        if let Ok(mut guard) = m.lock() {
            *guard = None;
        }
    }
}
