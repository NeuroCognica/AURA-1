# Gemini Chunk 5: AURA-1 Implementation Checklist

## Overview

This document provides a prioritized, actionable checklist for integrating Gemini Sentinel concepts into AURA-1. Each task includes file locations, implementation patterns, and verification steps.

---

## Phase 1: Schema Authority & Enforcement (High Priority)

### Task 1.1: Create Schema Lock Validation Script
**Estimated Effort:** 2-4 hours

**Files to Create:**
- `tools/schema_lock.py`

**Implementation:**
```python
# Three-layer validation:
# 1. Check canonical schema files exist (aura/schemas/)
# 2. Validate JSON Schema syntax (Draft-07+)
# 3. Compare OpenAPI components against canonical schemas
# 4. Run Rust parity binary and validate serialization output
```

**Dependencies:**
- Python 3.10+
- `pip install jsonschema`

**Verification:**
```bash
python tools/schema_lock.py
# Expected: "Schema lock: OK"
```

---

### Task 1.2: Create Rust Parity Binary
**Estimated Effort:** 3-4 hours

**Files to Create:**
- `backend/src/bin/schema_dump.rs`
- Update `backend/Cargo.toml` to register binary

**Implementation:**
```rust
// Serialize actual backend structs (CouncilMsg, ConstitutionalResponse, etc.)
// Print JSON to stdout for validation
[[bin]]
name = "schema_dump"
path = "src/bin/schema_dump.rs"
```

**Verification:**
```bash
cargo run --bin schema_dump -- envelope
# Expected: Valid JSON matching canonical schema
```

---

### Task 1.3: Add CI Schema Lock Workflow
**Estimated Effort:** 1-2 hours

**Files to Create:**
- `.github/workflows/schema-lock.yml`

**Implementation:**
```yaml
name: Schema Authority Lock
on: [push, pull_request]
jobs:
  constitutional-guard:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
      - run: pip install jsonschema
      - run: python tools/schema_lock.py
```

**Verification:**
- Push to GitHub
- Verify workflow runs and passes

---

### Task 1.4: OpenAPI Injection Pattern
**Estimated Effort:** 2-3 hours

**Files to Modify:**
- `backend/src/api.rs` (or wherever OpenAPI is generated)

**Implementation:**
```rust
// Read canonical schemas from files
// Inject directly into OpenAPI components
// Prevents utoipa drift
```

**Verification:**
```bash
# Generate OpenAPI
# Run schema_lock.py
# Expected: Parity check passes
```

---

## Phase 2: Intent Stratification Layer (Medium Priority)

### Task 2.1: Intent Classifier Module
**Estimated Effort:** 4-6 hours

**Files to Create:**
- `backend/orchestrator/src/intent_classifier.rs`
- Update `backend/orchestrator/src/lib.rs` to export module

**Implementation:**
```rust
pub struct IntentClassifier {
    action_verbs: Vec<String>,
    hypothetical_markers: Vec<String>,
}

impl IntentClassifier {
    pub fn classify(&self, input: &str) -> IntentClassification {
        // Categorize: Speculative vs Executable
        // Categorize: Reflective vs Directive
        // Categorize: Hypothetical vs Binding
        // Return: requires_sentinel bool
    }
}
```

**Test Cases:**
- "I wonder if I could delete my data" → Speculative, no Sentinel
- "Delete my conversation history" → Executable, requires Sentinel
- "What would happen if I deleted this?" → Hypothetical, no Sentinel

**Verification:**
```bash
cargo test intent_classifier
```

---

### Task 2.2: Integrate Intent Classifier into Pipeline
**Estimated Effort:** 2-3 hours

**Files to Modify:**
- `backend/orchestrator/src/lib.rs` (main orchestration logic)

**Implementation:**
```rust
pub async fn process_input(input: &str) -> ProcessedResponse {
    // 1. Classify intent
    let intent = intent_classifier.classify(input);
    
    // 2. Route based on classification
    if intent.requires_sentinel {
        constitutional_invoker.invoke(input).await
    } else {
        direct_llm_call(input).await
    }
}
```

**Verification:**
- Submit speculative query → verify no Sentinel log
- Submit executable command → verify Sentinel invoked

---

### Task 2.3: Semantic Claim Validator
**Estimated Effort:** 3-4 hours

**Files to Create:**
- `backend/orchestrator/src/claim_validator.rs`

**Implementation:**
```rust
pub struct ClaimValidator;

impl ClaimValidator {
    pub fn validate(
        response: &str, 
        context: &IntentClassification
    ) -> ClaimValidation {
        // Check for absolute language in speculative contexts
        // Flag authority claims in hypothetical contexts
        // Return violations list
    }
}
```

**Test Cases:**
- Speculative input + "definitely" in output → violation
- Hypothetical input + "will" in output → violation

**Verification:**
```bash
cargo test claim_validator
```

---

### Task 2.4: Memory Ontology Classifier
**Estimated Effort:** 3-4 hours

**Files to Create:**
- `backend/orchestrator/src/memory_ontology.rs`

**Implementation:**
```rust
pub enum MemoryType {
    Autobiographical,
    Structural,
    Symbolic,
    Derived,
    Immutable,
}

impl MemoryOntology {
    pub fn classify(memory_ref: &str) -> MemoryClassification {
        // Detect memory type from reference
        // Return mutability and deletion permissions
    }
}
```

**Verification:**
- "my conversation" → Autobiographical, deletion via crypto-shred only
- "system config" → Structural, immutable

---

## Phase 3: Mirrorborn Quiz System (High Priority)

### Task 3.1: QuizState with Weight Tracking
**Estimated Effort:** 3-4 hours

**Files to Create/Modify:**
- `backend/orchestrator/src/quiz/engine.rs`

**Implementation:**
```rust
pub struct QuizState {
    pub session_id: String,
    pub current_question: usize,
    pub current_probe: usize,
    pub weights: HashMap<String, f64>, // NEW
}

impl QuizState {
    pub fn add_features(&mut self, features: Vec<String>) { /* ... */ }
    pub fn get_accumulated_weights(&self) -> HashMap<String, f64> { /* ... */ }
}
```

**Verification:**
```bash
cargo test quiz_state::test_weight_accumulation
```

---

### Task 3.2: MirrorbornProfile Struct
**Estimated Effort:** 2-3 hours

**Files to Create:**
- `backend/orchestrator/src/quiz/profile.rs`

**Implementation:**
```rust
#[derive(Serialize, Deserialize)]
pub struct MirrorbornProfile {
    pub session_id: String,
    pub timestamp: String,
    pub archetypes: HashMap<String, f64>,
    pub primary_archetype: String,
    pub secondary_archetype: String,
}

impl MirrorbornProfile {
    pub fn new(session_id: &str, weights: HashMap<String, f64>) -> Self {
        // Normalize weights
        // Find top 2 archetypes
        // Return profile
    }
}
```

---

### Task 3.3: Obsidian Commit in Quiz Handler
**Estimated Effort:** 2-3 hours

**Files to Modify:**
- Quiz answer handler (wherever quiz answers are processed)

**Implementation:**
```rust
// Before advancing state:
ledger::append_event(
    &session_id,
    "user",
    EventType::QuizEntry,
    serde_json::json!({
        "question_index": state.current_question,
        "probe_index": state.current_probe,
        "user_answer": answer_text,
    })
)?;

// On completion:
let profile = MirrorbornProfile::new(&session_id, state.get_accumulated_weights());
std::fs::write(
    format!("data/profiles/profile_{}.json", session_id),
    serde_json::to_string_pretty(&profile)?
)?;
```

**Verification:**
- Complete quiz
- Verify `profile_{session}.json` created
- Verify ledger contains QuizEntry events

---

### Task 3.4: ProfileGenerated Event Type
**Estimated Effort:** 1 hour

**Files to Modify:**
- Event type enum (wherever EventType is defined)

**Implementation:**
```rust
pub enum EventType {
    // ... existing variants
    QuizEntry,
    ProfileGenerated,
}
```

---

### Task 3.5: Mirrorborn Verification Script
**Estimated Effort:** 2-3 hours

**Files to Create:**
- `tools/verify_mirrorborn.py`

**Implementation:**
```python
def run_quiz_speedrun():
    session_id = f"verify_{int(time.time())}"
    # Post to /api/quiz/start
    # Loop 240 times posting answers
    # Verify completion

def audit_artifacts():
    # Check profile JSON exists
    # Validate structure
    # Check ledger for ProfileGenerated event
```

**Verification:**
```bash
# Terminal 1: Start AURA backend
cargo run

# Terminal 2: Run verifier
python tools/verify_mirrorborn.py
# Expected: [SUCCESS] Mirrorborn Integration Verified
```

---

## Phase 4: Server Architecture Fixes (Critical)

### Task 4.1: Fix Accept Loop Pattern
**Estimated Effort:** 2-3 hours

**Files to Modify:**
- Main server startup code

**Current (Broken):**
```rust
loop {
    let (socket, addr) = listener.accept().await?;
    println!("Accepted {}", addr);
    // Socket dropped here!
}
```

**Fixed:**
```rust
pub async fn start_server(port: u16) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let app = create_router(); // Contains all routes + middleware
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Verification:**
```bash
cargo run
# In another terminal:
curl http://localhost:8080/api/health
# Expected: 200 OK response (not connection reset)
```

---

### Task 4.2: Sentinel Middleware Layer
**Estimated Effort:** 2-3 hours

**Files to Create:**
- Middleware function for request auditing

**Implementation:**
```rust
async fn sentinel_audit_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Log request to ledger
    ledger::append_event(
        "system",
        EventType::RequestReceived,
        serde_json::json!({
            "method": req.method().as_str(),
            "path": req.uri().path(),
        })
    )?;
    
    Ok(next.run(req).await)
}

// In router:
Router::new()
    .route("/api/quiz/answer", post(quiz_answer))
    .layer(middleware::from_fn(sentinel_audit_middleware))
```

**Verification:**
- Make API request
- Check ledger for RequestReceived event

---

## Phase 5: Integration & Testing (Medium Priority)

### Task 5.1: Profile → Archetype Tuning Integration
**Estimated Effort:** 4-6 hours

**Files to Create:**
- Profile watcher service (could be separate binary or module)

**Implementation:**
```rust
// Watch data/profiles/ directory
// On new profile JSON:
//   - Load profile
//   - Update archetype weights in orchestrator config
//   - Log ProfileLoaded event
```

**Verification:**
- Complete quiz
- Verify archetype behavior changes for that session

---

### Task 5.2: Integration Test Suite
**Estimated Effort:** 4-6 hours

**Files to Create:**
- `backend/tests/integration/test_full_stack.rs`

**Test Cases:**
```rust
#[tokio::test]
async fn test_quiz_ledger_persistence() {
    // Submit quiz answer
    // Query ledger for QuizEntry
    assert!(ledger_contains_quiz_entry);
}

#[tokio::test]
async fn test_profile_generation() {
    // Complete 240-probe quiz
    // Verify profile artifact exists
    // Verify ProfileGenerated event
}

#[tokio::test]
async fn test_intent_classification_routing() {
    // Submit speculative query
    // Verify no Sentinel invocation
    // Submit executable command
    // Verify Sentinel invoked
}
```

---

## Implementation Priority Matrix

| Phase | Priority | Effort | Dependencies | Impact |
|-------|----------|--------|--------------|--------|
| 1.1-1.4 Schema Lock | **HIGH** | 8-13h | None | Prevents drift, CI enforcement |
| 2.1-2.2 Intent Classifier | **MEDIUM** | 6-9h | None | Reduces Sentinel load |
| 3.1-3.5 Mirrorborn | **HIGH** | 10-15h | Schema Lock | User profiling |
| 4.1-4.2 Server Fix | **CRITICAL** | 4-6h | None | System stability |
| 2.3-2.4 Semantic/Memory | **LOW** | 6-8h | Intent Classifier | Enhanced validation |
| 5.1-5.2 Integration | **MEDIUM** | 8-12h | Mirrorborn, Intent | End-to-end flow |

---

## Week 1 Sprint Plan

**Day 1-2:**
- Task 4.1: Fix server accept loop (CRITICAL)
- Task 1.1: Create schema_lock.py
- Task 1.2: Create schema_dump binary

**Day 3:**
- Task 1.3: Add CI workflow
- Task 1.4: OpenAPI injection
- Verify Phase 1 complete

**Day 4-5:**
- Task 3.1: QuizState weights
- Task 3.2: MirrorbornProfile struct
- Task 3.3: Obsidian commit

**Day 6:**
- Task 3.4: ProfileGenerated event
- Task 3.5: Verification script
- Verify Phase 3 complete

**Day 7:**
- Task 2.1: Intent classifier
- Task 2.2: Pipeline integration
- Integration testing

---

## Success Criteria

### Phase 1: Schema Authority
- ✅ `schema_lock.py` runs without errors
- ✅ CI workflow blocks PRs with schema drift
- ✅ Rust binary serialization matches canonical schemas

### Phase 3: Mirrorborn
- ✅ Quiz answers persist to ledger immediately
- ✅ Profile artifacts generated on completion
- ✅ `verify_mirrorborn.py` passes end-to-end test

### Phase 4: Server
- ✅ `curl http://localhost:8080/api/health` returns 200 OK
- ✅ No WinError 10054 connection resets
- ✅ All requests logged to ledger

### Phase 2: Intent Stratification
- ✅ Speculative queries bypass Sentinel
- ✅ Executable commands invoke Sentinel
- ✅ Test suite validates routing logic

---

## Code Extraction Summary

**From Gemini Conversation → AURA-1:**

1. **Schema Lock Pattern**: Enforcement script + parity binary + CI
2. **Mirrorborn System**: Quiz engine + profile synthesis + obsidian commit
3. **Intent Classifier**: Pre-constitutional cognition layer
4. **Semantic Validator**: Post-LLM claim validation
5. **Memory Ontology**: Memory type classification before operations
6. **Server Fix**: Replace manual accept loop with standard serve
7. **Sentinel Middleware**: Request auditing layer

**Total Estimated Effort:** 50-70 hours for full implementation

**Recommended Sequence:** Phase 4 → Phase 1 → Phase 3 → Phase 2 → Phase 5
