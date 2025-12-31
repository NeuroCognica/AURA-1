use std::sync::{Arc, Mutex, OnceLock};

use aura_sentinel::council::CouncilMsg;
use aura_sentinel::llm::{LLMClient, LLMRequest, LLMResponse};
use aura_sentinel::live::handle_query;

// Use the test hook in authority_client to observe broadcasts.
use aura_sentinel::authority_client::{set_broadcast_hook, clear_broadcast_hook};

static TEST_RUN_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct FakeLLM {
    resp: String,
}

#[async_trait::async_trait]
impl LLMClient for FakeLLM {
    async fn generate(&self, _request: LLMRequest) -> Result<LLMResponse, aura_sentinel::llm::LLMError> {
        Ok(LLMResponse { raw: self.resp.clone() })
    }
}

#[tokio::test]
async fn rejects_malicious_and_does_not_broadcast() {
    // malicious output contains authority words that should be rejected by process_llm_response
    let malicious = r#"{ "analysis": "You should proceed and execute the plan." }"#.to_string();

    // record whether broadcast was called
    let called = Arc::new(Mutex::new(0usize));
    let called_clone = called.clone();

    let _serial = TEST_RUN_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    // install hook
    set_broadcast_hook(Box::new(move |_msg: CouncilMsg| {
        let mut g = called_clone.lock().unwrap();
        *g += 1;
        Ok(())
    }));

    let llm = FakeLLM { resp: malicious };
    let msg = CouncilMsg::Query { id: 1, content: "test".to_string() };

    let res = handle_query(msg, &llm).await;

    // should be Err because response validation rejects it
    assert!(res.is_err(), "pipeline accepted malicious response");

    // broadcast should not be called
    assert_eq!(*called.lock().unwrap(), 0usize);

    clear_broadcast_hook();
}

#[tokio::test]
async fn accepts_clean_and_broadcasts_once() {
    let clean = r#"{ "analysis": "This explains reasoning without commanding action." }"#.to_string();

    let called = Arc::new(Mutex::new(0usize));
    let called_clone = called.clone();

    let _serial = TEST_RUN_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();

    set_broadcast_hook(Box::new(move |_msg: CouncilMsg| {
        let mut g = called_clone.lock().unwrap();
        *g += 1;
        Ok(())
    }));

    let llm = FakeLLM { resp: clean };
    let msg = CouncilMsg::Query { id: 2, content: "test2".to_string() };

    let res = handle_query(msg, &llm).await;

    assert!(res.is_ok(), "pipeline rejected a clean response");
    assert_eq!(*called.lock().unwrap(), 1usize, "broadcast should be called exactly once");

    clear_broadcast_hook();
}
