# Phase 3: Mirrorborn Quiz Integration Plan

**Status:** READY TO BEGIN  
**Estimated:** 12-17 hours  
**Priority:** CRITICAL (Identity foundation for archetype matching)

---

## Current State Analysis

### Existing Files

1. **Cargo.toml** - Standalone package with Axum, RocksDB, Serde, UUID, Chrono
2. **main.rs** - Axum server with 3 endpoints:
   - POST `/submit_answer` - Submit probe answer (upserts, triggers profile gen at 240)
   - GET `/session/:session_id` - Get session answers + count
   - POST `/generate/:session_id` - Force profile generation
3. **store.rs** - RocksDB store with:
   - `upsert_answer()` - Idempotent answer storage (probe_id → index mapping)
   - `get_session_answers()` - Retrieve all answers for session
   - `answered_count()` - Count of answered probes
4. **quiz.rs** - Constants: `TOTAL_PROBES = 240`, helper functions
5. **mirrorborn.rs** - Profile generation:
   - `MirrorbornProfile` struct (session_id, answers, provenance)
   - `generate_profile()` - Writes atomic JSON to artifacts dir
6. **mirrorborn_vectors.rs** - Schema definitions (443 lines):
   - VectorId, QuizId, ModuleId, QuestionId, ProbeId types
   - ProbeType enum (ConcreteRecall, InternalProcess, MeaningReflection)
   - Raw expression layer structure (immutable)
7. **master_quiz.md** - 80 questions × 3 probes = 240 probes (3489 lines):
   - Explicit probe → schema feature mappings
   - TemporalReference, AgencyMarkers, EmotionalSurface, etc.
   - Each probe lists structural feature buckets

### Integration Gaps (What's Missing)

1. ❌ **Weight Accumulation**: Answers stored but not analyzed for cognitive features
2. ❌ **Feature Extraction**: Probe → schema mappings in MD, not in code
3. ❌ **Archetype Synthesis**: Profile contains raw answers, not archetype scores
4. ❌ **Obsidian Commit**: No ledger integration (Forever Law not enforced)
5. ❌ **Workspace Integration**: Standalone package, not part of AURA workspace
6. ❌ **Backend Communication**: No link to backend/orchestrator archetype system

---

## Architecture Decision: Quiz Service Design

### Option A: Standalone Microservice (Current)
- ✅ **Pros**: Isolated, clean API, easy to test independently
- ❌ **Cons**: No shared RocksDB, no ledger integration, separate process
- ❌ **Forever Law Risk**: Cannot enforce Obsidian Commit without backend ledger

### Option B: Backend Module Integration (Recommended)
- ✅ **Pros**: Shared RocksDB, direct ledger access, Obsidian Commit enforceable
- ✅ **Pros**: Type sharing (CouncilVerdict, ProfileGenerated events)
- ✅ **Pros**: Single process, no network hops
- ❌ **Cons**: Tighter coupling, need careful module boundaries

**DECISION: Option B** - Integrate as backend module for ledger access and type safety.

---

## Implementation Plan

### Task 3.1: Workspace Integration (2 hours)

**Goal:** Make mirrorborn_quiz a backend module or workspace sibling with backend access.

**Files:**
- `/Cargo.toml` (workspace root) - Add mirrorborn_quiz as member
- `/backend/Cargo.toml` - Add mirrorborn_quiz as dependency OR
- Move quiz files to `/backend/src/mirrorborn_quiz/` (recommended)

**Acceptance Criteria:**
- [ ] Quiz compiles as part of workspace
- [ ] Can import aura-backend types (CouncilVerdict, etc)
- [ ] No cyclic dependencies
- [ ] Store can access backend RocksDB path

**Decision Point:** Module vs Sibling Crate?
- **Module** (`backend/src/mirrorborn_quiz/mod.rs`): Direct ledger access, simpler
- **Sibling Crate** (`mirrorborn_quiz/`): Better separation, but needs RPC or shared DB

**Recommendation:** Backend module for Phase 3, extract to microservice in Phase 4 if needed.

---

### Task 3.2: Weight Tracking in QuizState (3-4 hours)

**Goal:** Parse master_quiz.md probe mappings and accumulate cognitive features from answers.

**Files:**
- `backend/src/mirrorborn_quiz/quiz_engine.rs` (new)
- `backend/src/mirrorborn_quiz/feature_extraction.rs` (new)
- `backend/src/mirrorborn_quiz/probe_mappings.rs` (generated from master_quiz.md)

**Data Structures:**

```rust
pub struct QuizState {
    pub session_id: String,
    pub answered_count: u64,
    pub weights: HashMap<String, f64>, // "TemporalReference" → 0.73
    pub started_at_ms: i64,
    pub last_update_ms: i64,
}

impl QuizState {
    pub fn add_features(&mut self, features: &[(&str, f64)]) {
        for (feature, weight) in features {
            *self.weights.entry(feature.to_string()).or_insert(0.0) += weight;
        }
    }
    
    pub fn get_normalized_weights(&self) -> HashMap<String, f64> {
        let max = self.weights.values().cloned().fold(0.0, f64::max);
        if max == 0.0 { return self.weights.clone(); }
        self.weights.iter().map(|(k, v)| (k.clone(), v / max)).collect()
    }
}
```

**Probe Mappings Strategy:**

Option A: Manual mapping table (fast to implement):
```rust
pub fn extract_features(probe_id: &str, answer: &str) -> Vec<(&'static str, f64)> {
    match probe_id {
        "Q1_A" => vec![
            ("TemporalReference", 1.0),
            ("AgencyMarkers", 0.8),
            ("ConcreteRecall", 1.0),
        ],
        // 240 probes...
    }
}
```

Option B: Parse master_quiz.md at build time (cleaner, harder):
- Use build.rs to extract probe → feature mappings
- Generate probe_mappings.rs from markdown

**Recommendation:** Option A for Phase 3 MVP (manual first 10 probes, stub rest).

**Acceptance Criteria:**
- [ ] QuizState with weights HashMap
- [ ] add_features() accumulates correctly
- [ ] get_normalized_weights() returns 0.0-1.0 range
- [ ] Unit tests: multiple answers accumulate weights
- [ ] Feature extraction for at least 10 probes (Q1-Q10)

---

### Task 3.3: MirrorbornProfile Synthesis (2-3 hours)

**Goal:** Generate archetype profile from accumulated weights.

**Files:**
- `backend/src/mirrorborn_quiz/profile_synthesis.rs` (new)
- Update `mirrorborn.rs` MirrorbornProfile struct

**Archetype Mapping Logic:**

```rust
pub struct ArchetypeScore {
    pub archetype: String, // "Architect", "Explorer", etc.
    pub score: f64,        // 0.0-1.0
    pub confidence: f64,   // Statistical confidence
}

pub struct MirrorbornProfile {
    pub session_id: String,
    pub primary_archetype: ArchetypeScore,
    pub secondary_archetype: Option<ArchetypeScore>,
    pub feature_weights: HashMap<String, f64>, // Normalized
    pub provenance: Provenance,
    pub answered_count: u64,
    pub generated_at_ms: i64,
}

pub fn synthesize_profile(state: &QuizState) -> MirrorbornProfile {
    let weights = state.get_normalized_weights();
    
    // Map features → archetype affinities
    let architect_score = calculate_archetype_affinity(&weights, &ARCHITECT_FEATURES);
    let explorer_score = calculate_archetype_affinity(&weights, &EXPLORER_FEATURES);
    // ... all 7 archetypes
    
    let (primary, secondary) = rank_archetypes(all_scores);
    
    MirrorbornProfile {
        session_id: state.session_id.clone(),
        primary_archetype: primary,
        secondary_archetype: secondary,
        feature_weights: weights,
        // ...
    }
}
```

**Archetype Feature Mappings (from archetypes/):**

```rust
const ARCHITECT_FEATURES: &[&str] = &[
    "TemporalReference", "CoherenceMetrics", "SensoryDetail", // Structure
];

const EXPLORER_FEATURES: &[&str] = &[
    "AgencyMarkers", "RegulationMarkers", "EmotionalSurface", // Openness
];

// ... 5 more archetypes
```

**Acceptance Criteria:**
- [ ] MirrorbornProfile struct with archetype scores
- [ ] synthesize_profile() generates from QuizState
- [ ] Primary archetype identified (highest score)
- [ ] Secondary archetype (2nd highest, if > threshold)
- [ ] Feature mappings for all 7 archetypes
- [ ] Unit test: known weights → expected archetype

---

### Task 3.4: Obsidian Commit Pattern (2-3 hours)

**Goal:** Enforce Forever Law: append-only ledger before state mutation.

**Files:**
- `backend/src/mirrorborn_quiz/ledger.rs` (new)
- Update `main.rs` submit_handler to use ledger

**Ledger Event Schema:**

```rust
#[derive(Serialize, Deserialize)]
pub struct QuizAnswerEvent {
    pub session_id: String,
    pub probe_id: String,
    pub answer: String,
    pub features_extracted: Vec<(String, f64)>,
    pub timestamp_ms: i64,
    pub ledger_idx: u64, // MMR index
}

#[derive(Serialize, Deserialize)]
pub struct ProfileGeneratedEvent {
    pub session_id: String,
    pub profile_path: String,
    pub primary_archetype: String,
    pub answered_count: u64,
    pub timestamp_ms: i64,
    pub ledger_idx: u64,
}
```

**Integration with Backend Ledger:**

```rust
// In submit_handler BEFORE upsert_answer:
let features = extract_features(&payload.probe_id, &payload.answer);

let event = QuizAnswerEvent {
    session_id: payload.session_id.clone(),
    probe_id: payload.probe_id.clone(),
    answer: payload.answer.clone(),
    features_extracted: features.clone(),
    timestamp_ms: Utc::now().timestamp_millis(),
    ledger_idx: 0, // Set by ledger
};

// CRITICAL: Ledger write MUST succeed before state update
let ledger_idx = state.store.append_ledger_event(&event)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Ledger write failed: {}", e)))?;

// Only AFTER ledger commit:
state.store.upsert_answer(&payload.session_id, &payload.probe_id, &payload.answer)?;
state.quiz_state.add_features(&features);
```

**Forever Law Enforcement:**
- ✅ **Immutability**: Ledger entries never mutate
- ✅ **Fail-Closed**: Request fails if ledger write fails
- ✅ **Causality**: State change only after ledger commit
- ✅ **Auditability**: Every answer has ledger provenance

**Acceptance Criteria:**
- [ ] append_ledger_event() writes to RocksDB with MMR integrity
- [ ] submit_handler rejects request if ledger write fails
- [ ] Features extracted BEFORE ledger write (deterministic)
- [ ] ProfileGeneratedEvent appended on completion
- [ ] Integration test: ledger failure → no state change

---

### Task 3.5: Profile Generation on Completion (2-3 hours)

**Goal:** Detect completion (240 probes), synthesize profile, write artifact, emit event.

**Files:**
- Update `submit_handler` in main.rs
- `backend/src/mirrorborn_quiz/artifacts.rs` (new)

**Completion Detection:**

```rust
// In submit_handler after state update:
let cnt = state.store.answered_count(&payload.session_id)?;

if cnt as usize >= TOTAL_PROBES {
    let quiz_state = state.quiz_state_store.get(&payload.session_id)?;
    
    // Synthesize profile
    let profile = synthesize_profile(&quiz_state);
    
    // Write artifact atomically
    let artifact_path = write_profile_artifact(&profile, &state.artifacts_dir)?;
    
    // Append ProfileGenerated event to ledger
    let event = ProfileGeneratedEvent {
        session_id: payload.session_id.clone(),
        profile_path: artifact_path.display().to_string(),
        primary_archetype: profile.primary_archetype.archetype.clone(),
        answered_count: cnt,
        timestamp_ms: Utc::now().timestamp_millis(),
        ledger_idx: 0,
    };
    
    state.store.append_ledger_event(&event)?;
    
    // Broadcast to council channel (optional)
    let _ = state.council_bcast.send(serde_json::to_string(&profile)?);
}
```

**Artifact Storage:**

```rust
pub fn write_profile_artifact(profile: &MirrorbornProfile, artifacts_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(artifacts_dir)?;
    
    let filename = format!("profile_{}.json", profile.session_id);
    let tmp_path = artifacts_dir.join(format!("{}.tmp", filename));
    let final_path = artifacts_dir.join(&filename);
    
    // Write to temp file
    let json = serde_json::to_vec_pretty(&profile)?;
    fs::write(&tmp_path, &json)?;
    
    // Atomic rename
    fs::rename(&tmp_path, &final_path)?;
    
    Ok(final_path)
}
```

**Acceptance Criteria:**
- [ ] Completion detected at probe #240
- [ ] Profile synthesized from accumulated weights
- [ ] Artifact written to data/profiles/ atomically
- [ ] ProfileGenerated event in ledger
- [ ] Integration test: 240 answers → profile artifact exists

---

### Task 3.6: Testing & Validation (2-3 hours)

**Test Suite:**

1. **Unit Tests:**
   - `test_weight_accumulation()` - Features accumulate correctly
   - `test_weight_normalization()` - Normalize to 0.0-1.0
   - `test_archetype_synthesis()` - Known weights → expected archetype
   - `test_obsidian_commit_failure()` - Ledger failure prevents state change

2. **Integration Tests:**
   - `test_full_quiz_flow()` - 240 answers → profile generated
   - `test_profile_artifact()` - Artifact written atomically
   - `test_ledger_events()` - 240 answer events + 1 profile event
   - `test_idempotent_answers()` - Re-answering probe updates, doesn't duplicate

3. **Load Tests:**
   - Multiple concurrent sessions
   - Verify ledger ordering
   - Check RocksDB performance

**Acceptance Criteria:**
- [ ] All unit tests pass
- [ ] Integration test: full 240-probe quiz completes
- [ ] Ledger integrity verified (no gaps, monotonic indices)
- [ ] Profile artifact content validated

---

### Task 3.7: Documentation (1 hour)

**Files:**
- Update `README.md` with new architecture
- Add `MIRRORBORN_SPEC.md` with archetype mapping logic
- Document API changes
- Add developer guide for adding new features/archetypes

---

## Phase 3 Success Metrics

✅ **Functional:**
- [ ] 240 probe answers → complete profile
- [ ] Primary archetype identified correctly (manual validation)
- [ ] Profile artifact written atomically
- [ ] Obsidian Commit enforced (Forever Law)

✅ **Architectural:**
- [ ] Integrated with backend RocksDB
- [ ] Ledger events for every answer + profile generation
- [ ] No cyclic dependencies
- [ ] Clean module boundaries

✅ **Testing:**
- [ ] Unit test coverage > 80%
- [ ] Integration tests pass
- [ ] Load test: 10 concurrent sessions

---

## Phase 3 Risks & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Master_quiz.md parsing complexity | HIGH | MEDIUM | Manual mapping table for MVP (first 10 probes) |
| Archetype feature mappings unknown | MEDIUM | HIGH | Review archetypes/*.json for canonical features |
| Ledger integration cyclic dependency | LOW | HIGH | Move quiz to backend module, not separate crate |
| RocksDB contention (quiz + backend) | LOW | MEDIUM | Use separate column families for quiz data |
| Profile synthesis accuracy | MEDIUM | HIGH | Manual validation with test subjects (Phase 4) |

---

## Next Steps (Immediate Actions)

1. **Move mirrorborn_quiz files to backend/src/mirrorborn_quiz/**
2. **Create backend/src/mirrorborn_quiz/mod.rs** with module exports
3. **Add QuizState struct with weights HashMap**
4. **Implement feature extraction for Q1-Q10** (manual mapping)
5. **Test weight accumulation** with 30 answers (Q1-Q10 × 3 probes)

---

## Post-Phase 3 Integration Points

**Backend APIs to Call:**
- `POST /api/mirrorborn/submit_answer` - Submit probe answer
- `GET /api/mirrorborn/session/:id` - Get session progress
- `GET /api/mirrorborn/profile/:session_id` - Retrieve generated profile

**Frontend Integration:**
- Quiz UI consumes `/api/mirrorborn` endpoints
- Display progress (answered_count / 240)
- Show generated profile on completion

**Archetype Matching:**
- Profile primary_archetype → activate matching archetype in UI
- Load archetype-specific system prompt in ConstitutionalInvoker
- Adjust UI theme based on archetype

---

**Status:** PLAN COMPLETE - Ready to begin implementation  
**Start with:** Task 3.1 (Workspace Integration)
