# Gemini Chunk 4: Integration Architecture & Server Fixes

## Multi-Repo Stack Structure

### Repository Roles

**sentinel-core** (Rust)
- Event-sourced security substrate
- Append-only ledger with SHA-256 hash chain
- Identity & capability management
- HTTP API with guard middleware
- Crates: sentinel_api, sentinel_store, sentinel_identity, sentinel_capabilities, sentinel_cli

**aura** (Schemas + UI)
- Canonical JSON schemas (single source of truth)
- `CanonicalEnvelope.json`, `execution_proof.schema.json`
- Test fixtures
- Client UI (package.json)

**mecha** (Orchestration)
- Archetype LLM system
- Design/operational docs
- Python orchestration layer
- Chat interface

**senkern** (Quiz System)
- Mirrorborn quiz administration
- User login/profile creation
- Currently RAM-only (needs ledger integration)

### Integration Gaps Identified

1. **Senkern → Sentinel-Core**: Quiz data not persisting to ledger
2. **Profile → Mecha**: Generated profiles not feeding archetype tuning
3. **Schema Drift**: OpenAPI and Rust structs diverging from canonical schemas

## Server Connection Abort Fix

### Problem Diagnosis

**Symptoms:**
- WinError 10054 (Connection forcibly closed by remote host)
- Server logs show: "Accepted connection" → immediately "About to call accept" again
- Socket dropped before handler spawned

**Root Cause:**
```rust
// BAD CODE (Accept-and-Drop Pattern)
loop {
    let (socket, addr) = listener.accept().await.unwrap();
    println!("Accepted connection from {}", addr);
    // ERROR: socket goes out of scope here, Drop closes connection
}
```

### Solution Pattern

**Option A: Use Standard Axum/Hyper Serve**
```rust
// CORRECT: Library-managed serving
pub async fn start_server(state: AppState, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let app = create_router(state); // Contains Sentinel middleware
    
    // Axum 0.7+
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

**Option B: Manual Loop with Task Spawning**
```rust
// CORRECT: Spawn handler task
pub async fn start_server_manual(state: AppState, port: u16) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let app = create_router(state);
    
    loop {
        let (socket, remote_addr) = listener.accept().await?;
        println!("Accepted connection from {}", remote_addr);
        
        let service = app.clone();
        
        // CRITICAL: Spawn task so socket stays alive
        tokio::spawn(async move {
            if let Err(err) = hyper::server::conn::Http::new()
                .serve_connection(socket, service)
                .await 
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}
```

### Ensuring Sentinel Route Enforcement

**Router Construction Must Include Middleware:**
```rust
use axum::{Router, middleware};

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/quiz/start", post(quiz_start))
        .route("/api/quiz/answer", post(quiz_answer))
        .route("/__debug/ping", get(debug_ping))
        // CRITICAL: Sentinel middleware layer
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sentinel_guard_middleware
        ))
        .with_state(state)
}

async fn sentinel_guard_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Audit request to conversation_ledger (Forever Law)
    let _ = conversation_ledger::append_event(
        "system",
        "system",
        EventType::RequestReceived,
        serde_json::json!({
            "method": req.method().as_str(),
            "path": req.uri().path(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    );
    
    // Continue to handler
    Ok(next.run(req).await)
}
```

## Senkern → Sentinel-Core Integration

### Current State
- Senkern quiz stores to RAM (`AppState.quiz_sessions: Arc<Mutex<HashMap>>`)
- Profile generation works but volatile
- No persistence to sentinel-core ledger

### Integration Architecture

**Ledger Event Flow:**
```
senkern/quiz/answer → append_event(QuizEntry) 
                   → sentinel-core/event_store 
                   → SHA-256 hash chain
```

**Profile Flow:**
```
senkern/profile_synthesis → write_artifact(profile.json)
                          → append_event(ProfileGenerated)
                          → mecha/watch_for_profile
                          → archetype_tuning(profile)
```

### Implementation Steps

**1. Shared Event Type Definitions**

Create shared event schema in `aura/schemas/events.json`:
```json
{
  "QuizEntry": {
    "type": "object",
    "properties": {
      "session_id": { "type": "string", "format": "uuid" },
      "question_index": { "type": "integer" },
      "probe_index": { "type": "integer" },
      "user_answer": { "type": "string" },
      "validation_score": { "type": "number" }
    },
    "required": ["session_id", "question_index", "probe_index", "user_answer"]
  },
  "ProfileGenerated": {
    "type": "object",
    "properties": {
      "session_id": { "type": "string", "format": "uuid" },
      "profile_path": { "type": "string" },
      "primary_archetype": { "type": "string" },
      "timestamp": { "type": "string", "format": "date-time" }
    },
    "required": ["session_id", "profile_path", "primary_archetype"]
  }
}
```

**2. Unified Ledger Client**

`backend/shared/src/ledger_client.rs`:
```rust
use reqwest::Client;
use serde_json::Value;

pub struct LedgerClient {
    sentinel_url: String,
    client: Client,
}

impl LedgerClient {
    pub fn new(sentinel_url: String) -> Self {
        Self {
            sentinel_url,
            client: Client::new(),
        }
    }
    
    pub async fn append_event(
        &self,
        actor: &str,
        event_type: &str,
        payload: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let envelope = create_canonical_envelope(actor, event_type, payload)?;
        
        self.client
            .post(format!("{}/api/events/append", self.sentinel_url))
            .json(&envelope)
            .send()
            .await?
            .error_for_status()?;
        
        Ok(())
    }
}
```

**3. Senkern Integration Point**

Update `senkern/src/web.rs`:
```rust
pub async fn quiz_answer(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuizAnswerRequest>,
) -> impl IntoResponse {
    // ... validation logic ...
    
    if validation.is_valid {
        // Write to sentinel-core ledger instead of local file
        state.ledger_client.append_event(
            &payload.session_id,
            "QuizEntry",
            serde_json::json!({
                "session_id": payload.session_id,
                "question_index": quiz_state.current_question,
                "probe_index": quiz_state.current_probe,
                "user_answer": payload.answer_text,
                "validation_score": validation.score
            })
        ).await?;
        
        // ... rest of handler ...
    }
}
```

**4. Mecha Profile Consumer**

`mecha/src/profile_watcher.py`:
```python
import json
import time
from pathlib import Path
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class ProfileHandler(FileSystemEventHandler):
    def on_created(self, event):
        if event.src_path.endswith('.json') and 'profile_' in event.src_path:
            self.process_profile(event.src_path)
    
    def process_profile(self, path):
        with open(path) as f:
            profile = json.load(f)
        
        # Update archetype weights for user
        session_id = profile['session_id']
        primary = profile['primary_archetype']
        
        # Send to archetype tuning system
        update_archetype_config(session_id, profile['archetypes'])
        print(f"✓ Profile loaded: {session_id} → {primary}")

def watch_profiles():
    observer = Observer()
    handler = ProfileHandler()
    observer.schedule(handler, path="./codex/artifacts", recursive=False)
    observer.start()
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()
```

## Verification Testing

**Integration Test Script:**
```python
#!/usr/bin/env python3
# test_full_stack.py

def test_quiz_persistence():
    """Verify quiz answers persist to sentinel-core ledger"""
    session_id = start_quiz_session()
    submit_quiz_answer(session_id, "Test answer")
    
    # Query sentinel-core for event
    events = query_ledger(session_id, event_type="QuizEntry")
    assert len(events) > 0, "Quiz answer not persisted"

def test_profile_generation():
    """Verify profile artifact created and ledger event emitted"""
    session_id = complete_quiz_speedrun()
    time.sleep(2)
    
    # Check artifact
    profile_path = f"./codex/artifacts/profile_{session_id}.json"
    assert os.path.exists(profile_path), "Profile artifact missing"
    
    # Check ledger event
    events = query_ledger(session_id, event_type="ProfileGenerated")
    assert len(events) == 1, "ProfileGenerated event missing"

def test_mecha_integration():
    """Verify mecha receives and processes profile"""
    session_id = complete_quiz_speedrun()
    time.sleep(5)  # Allow mecha watcher to process
    
    # Check archetype config updated
    config = load_archetype_config(session_id)
    assert config is not None, "Mecha did not process profile"
```

## Key Takeaways

1. **Server Fix**: Replace manual accept loops with standard serve patterns
2. **Sentinel Middleware**: All requests must flow through audit layer
3. **Ledger Integration**: Quiz data persists to sentinel-core event store
4. **Profile Pipeline**: Artifact → Event → Mecha watch → Archetype tuning
5. **Unified Client**: Shared ledger_client for consistent event submission
