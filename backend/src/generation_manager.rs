use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use std::future::Future;

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
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_new(&self, sid: &str) -> (String, CancellationToken) {
        let mut map = self.inner.lock().await;

        if let Some(old) = map.remove(sid) {
            old.cancel.cancel();
        }

        let gen_id = format!("g-{}", Uuid::new_v4());
        let cancel = CancellationToken::new();

        map.insert(
            sid.to_string(),
            ActiveGen {
                gen_id: gen_id.clone(),
                cancel: cancel.clone(),
            },
        );
        (gen_id, cancel)
    }

    /// Spawn a detached generation task owned by the manager.
    ///
    /// The provided function `f` will be called with `(gen_id, cancel)`
    /// inside the spawned task. Returns the generated `gen_id`.
    pub async fn spawn_generation<F, Fut>(&self, session_id: &str, f: F) -> String
    where
        F: FnOnce(String, CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (gen_id, cancel) = self.start_new(session_id).await;
        let gen_id_clone = gen_id.clone();

        // Spawn the provided future so generation runs in background.
        tokio::spawn(async move {
            f(gen_id_clone, cancel).await;
        });

        gen_id
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
