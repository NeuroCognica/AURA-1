use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub struct GenerationManager {
    inner: Arc<Mutex<HashMap<String, ActiveGen>>>,
}

struct ActiveGen {
    #[allow(dead_code)]
    gen_id: String,
    cancel: CancellationToken,
}

impl GenerationManager {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub async fn start_new(&self, sid: &str) -> (String, CancellationToken) {
        let mut map = self.inner.lock().await;

        if let Some(old) = map.remove(sid) {
            old.cancel.cancel();
        }

        let gen_id = format!("g-{}", Uuid::new_v4());
        let cancel = CancellationToken::new();

        map.insert(sid.to_string(), ActiveGen { gen_id: gen_id.clone(), cancel: cancel.clone() });
        (gen_id, cancel)
    }

    pub async fn cancel(&self, sid: &str) {
        let mut map = self.inner.lock().await;
        if let Some(active) = map.remove(sid) {
            active.cancel.cancel();
        }
    }

    pub async fn current_gen_id(&self, sid: &str) -> Option<String> {
        let map = self.inner.lock().await;
        map.get(sid).map(|a| a.gen_id.clone())
    }
}
