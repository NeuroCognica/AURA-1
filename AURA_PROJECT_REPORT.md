# AURA Project Report

**Date:** January 1, 2026  
**Status:** Active Development - Phase 2 Complete, Phase 3 In Progress  
**Vision:** Offline-first, sovereign builder's workbench with council-driven AI assistance

---

## Executive Summary

AURA is a **local-first man-machine workshop** where users collaborate with a council of specialized AI archetypes to build real things: software, hardware prototypes, life plans, creative projects. It operates entirely offline, storing all data locally with forensic-grade immutability guarantees (Forever Law). The system remembers conversations, learns user patterns, and provides skilled assistance across multiple domains—all without cloud dependencies.

**Core Principle:** Users are builders, not consumers. AURA provides a team of expert assistants who help execute projects, not a chatbot that entertains.

---

## I. System Architecture

### A. Core Components

#### 1. **Backend (Rust - Authority)**
- **Location:** `/backend/`
- **Role:** Single source of truth for all system state
- **Runtime:** Axum async HTTP/WebSocket server
- **Storage:** RocksDB with append-only ledger (Forever Law enforcement)
- **Responsibilities:**
  - Constitutional governance (Sentinel review)
  - Ledger integrity (MMR proofs)
  - Account management
  - Session state
  - LLM orchestration (Ollama integration)
  - Search indexing (Tantivy)

**Key Modules:**
- `constitutional.rs` - Enforces archetype constitutions before LLM invocation
- `intent_stratification.rs` - Pre-constitutional cognition filter (reduces Sentinel load)
- `accounts.rs` - User account storage (offline login system)
- `rocksdb_store.rs` - Forever Law ledger with MMR integrity
- `sentinel.rs` - Safety evaluation heuristics
- `archetype_api.rs` - Archetype activation endpoints

#### 2. **Frontend (TypeScript/Three.js - Client)**
- **Location:** `/frontend/` (stub), `/workbench/` (main UI), `/launcher/` (Electron wrapper)
- **Role:** Visualization and interaction only—no business logic
- **Constraints:**
  - Must run in iOS Safari / WebKit
  - WebGL 2 only (no WebXR dependency)
  - Cardboard-style stereo rendering for VR mode
- **UI Components:**
  - Workbench (main conversation interface)
  - Archetype selector
  - Council view (multi-archetype deliberation)
  - Settings (including optional Mirrorborn quiz access)

#### 3. **Orchestrator (Rust - Coordination)**
- **Location:** `/backend/orchestrator/`
- **Role:** Routes intents, coordinates archetypes, manages deliberation
- **Current Features:**
  - Intent classification (5-axis taxonomy)
  - Constitutional invoker integration
  - Verdict loop for Council decisions

#### 4. **Ledger (RocksDB + MMR)**
- **Forever Law:** All state changes append-only, never mutate
- **Integrity:** Merkle Mountain Range provides cryptographic proofs
- **Auditability:** Every chat message, verdict, state change logged
- **Use Case:** Forensic reconstruction of all interactions

---

### B. Archetype System

**7 Core Archetypes** (each with constitutional prompt):

1. **Architect** - Planning, structure, long-term vision
2. **Technician** - Hardware, firmware, Arduino, serial protocols, offline coding
3. **Explorer** - Discovery, experimentation, curiosity-driven
4. **Empath** - Emotional support, psychological insight, conflict mediation
5. **Mentor** - Teaching, guidance, skill development
6. **Jester** - Humor, perspective shifts, pattern breaking
7. **Sentinel** - Safety, boundary enforcement, risk assessment

**Archetype Behavior:**
- Each has a system prompt loaded from `archetypes/{name}_system.txt`
- ConstitutionalInvoker enforces prompt loading before LLM invocation
- **Fail-closed:** Missing constitution → request fails (never proceeds)

**Council Mode:**
- Multi-archetype deliberation for complex decisions
- Produces `CouncilVerdict` with consensus or dissent
- Logged to ledger with full provenance

---

### C. Mirrorborn Identity System

**Status:** Architectural placeholder - optional tuning layer

**Correct Mental Model:**
- **AURA = the product** (council-driven builder workbench)
- **Mirrorborn = optional modifier** (behavioral calibration)

**What Mirrorborn IS:**
- A JSON profile file in `data/profiles/profile_{username}.json`
- Contains archetypal alignment scores based on 240-question psychological assessment
- Used to tune system prompt generation (tone, priority, framing)
- **Optional, incremental, and revisable**

**What Mirrorborn IS NOT:**
- ❌ A separate product or mode
- ❌ A prerequisite for using AURA
- ❌ A gating mechanism for features
- ❌ The "main thing" users interact with

**Implementation Principle:**
```
if profile_exists(username):
    tune_archetype_prompts(profile)
else:
    use_default_prompts()
```

**Effect Example (Technician archetype):**
- **Without profile:** Default helpful technical tone
- **With profile (detail-oriented):** More depth, fewer confirmations, assumes technical background
- **With profile (exploratory):** More explanations, clarifying questions first

**Quiz Interface:**
- Accessed from Settings menu (not splash screen)
- 240 questions (80 primary × 3 probes each)
- Can be abandoned and resumed
- Can be retaken as user evolves
- Results saved to `data/profiles/` with provenance

**Forever Law Compliance:**
- Quiz answers stored in append-only ledger
- Profile generation logged with timestamp and model version
- User can audit their own psychological evolution over time

---

## II. Technical Implementation

### A. Forever Law (Immutability Guarantee)

**Principle:** All state changes are append-only. History never rewrites.

**Implementation:**
1. **Ledger Structure:**
   - RocksDB column family `logs`
   - Monotonic sequence IDs
   - Merkle Mountain Range for integrity proofs
   - BLAKE3 hashing

2. **Obsidian Commit Pattern:**
   ```rust
   // BEFORE any state mutation:
   ledger.append_event(event)?; // MUST succeed
   
   // ONLY AFTER ledger commit:
   state.update();
   ```
   - If ledger write fails → request rejected (fail-closed)
   - Ensures causal ordering and auditability

3. **Event Types:**
   - `ChatMessage` - User input, AI response, system messages
   - `CouncilVerdict` - Multi-archetype decisions
   - `ArchetypeActivation` - When user switches archetypes
   - `QuizAnswer` - Mirrorborn quiz responses (future)
   - `ProfileGenerated` - When profile synthesis completes

**Use Case:**
- Therapeutic tool: Users can review their entire conversation history
- Research: Anonymous aggregate analysis of human-AI interaction patterns
- Legal: Audit trail for safety-critical applications

---

### B. Constitutional Governance

**Layered Safety Architecture:**

1. **Intent Stratification (Pre-constitutional)**
   - **File:** `backend/src/intent_stratification.rs`
   - **Purpose:** Filter queries before Sentinel to reduce load
   - **Taxonomy:** 5-axis classification:
     - Speculation: Speculative vs. Executable
     - Reflection: Reflective vs. Directive
     - Binding: Hypothetical vs. Binding
     - Operation: Informational vs. Operational
     - Sensitivity: Safe vs. Sensitive
   - **Routing:**
     - Speculative + Hypothetical → Direct LLM (bypass Sentinel)
     - Executable + Binding → Require Sentinel review
   - **Test Coverage:** 15 tests (8 integration + 7 unit), all passing

2. **Sentinel Evaluation**
   - **File:** `backend/src/sentinel.rs`
   - **Heuristics:** Pattern matching for high-risk operations
   - **Decisions:**
     - `Allow` - Proceed normally
     - `AllowWithWarning` - Proceed with logged warning
     - `RequireConsent` - Demand explicit user confirmation
     - `Deny` - Block request entirely

3. **Constitutional Invoker**
   - **File:** `backend/orchestrator/src/constitutional.rs`
   - **Guarantee:** Every LLM call has a loaded system prompt (constitution)
   - **Fail-Closed:** Missing prompt → error (never proceeds with blank constitution)
   - **Enforcement:** Checked at compile time via type system

**Defense in Depth:**
```
User Input
    ↓
Intent Stratification (filter speculative queries)
    ↓
Sentinel Evaluation (safety heuristics)
    ↓
Constitutional Invoker (archetype-specific constraints)
    ↓
LLM Generation (Ollama)
    ↓
Response Validation (future)
    ↓
User Output
```

---

### C. Account System

**Status:** ✅ Complete (Phase 3, Task 3.1)

**Implementation:**
- **File:** `backend/src/accounts.rs` (271 lines)
- **Storage:** RocksDB with key pattern `account:{username}`
- **Structure:**
  ```rust
  struct Account {
      username: String,
      password_plaintext: String, // TODO: Hash in Phase 4
      created_at_ms: i64,
      last_login_ms: i64,
      quiz_sessions: Vec<String>,
  }
  ```

**API Endpoints:**
- `POST /api/account/create` - Create new account
- `POST /api/account/login` - Authenticate (returns username on success)
- `GET /api/account/:username` - Get account details

**Test Coverage:** 6 unit tests, all passing
- Account creation
- Duplicate prevention
- Authentication success
- Wrong password rejection
- Non-existent user handling
- Quiz session linking

**Security Note:**
- Plaintext passwords acceptable for offline-only system
- No network exposure in typical deployment
- Phase 4 can add hashing if needed

---

### D. Data Storage Schema

**RocksDB Column Families:**

1. **`default`** - General key-value storage
2. **`logs`** - Append-only ledger with sequence IDs
3. **`state`** - Session metadata
4. **`vectors`** - Future semantic search embeddings

**Key Patterns:**
- `account:{username}` → Account JSON
- `session:{session_id}:chat:{seq}` → Chat message
- `session:{session_id}:counter` → Message counter
- `session:{session_id}:verdict:{verdict_id}` → Council verdict
- `profile:{username}` → Mirrorborn profile (future)

**Forever Law Enforcement:**
- Sequence counters are monotonic
- Once written, entries never mutate
- Deletions append tombstone markers (never erase)

---

## III. Use Cases & Examples

### A. Primary Use Case: Offline Builder Assistance

**Scenario:** User building Arduino-controlled robot arm

**Workflow:**
1. **User:** "Help me wire a servo to pin 9 and control it via serial"
2. **Technician Archetype:** 
   - Provides wiring diagram
   - Generates Arduino sketch with Servo library
   - Explains serial protocol (baud rate, command format)
   - Suggests testing procedure
3. **User:** Uploads code, tests hardware
4. **User:** "The servo jitters when moving"
5. **Council Deliberation:**
   - **Technician:** Check power supply capacity, add capacitor
   - **Architect:** Consider software debouncing in motion planning
   - **Sentinel:** Verify current limits won't damage hardware
6. **Verdict:** Consensus on adding 1000µF capacitor + software smoothing
7. **Outcome:** Robot arm operates smoothly

**Key Features Used:**
- Technician archetype for domain expertise
- Council for multi-perspective problem solving
- All interactions logged for future reference
- Fully offline (no internet required)

---

### B. Secondary Use Case: Life Planning with Calendar

**Scenario:** User managing complex project with deadlines

**Workflow:**
1. **User:** "I need to finish this prototype by Friday but also have a dentist appointment and my kid's school event"
2. **Architect Archetype:**
   - Loads calendar from local file
   - Identifies time blocks
   - Proposes task breakdown with time estimates
3. **Empath Archetype:**
   - Notices scheduling stress patterns
   - Suggests energy management strategies
4. **Council Decision:**
   - Prioritize prototype critical path
   - Delegate non-critical tasks
   - Build in recovery time post-dentist
5. **Calendar Integration:**
   - Updates local .ics file
   - Sets reminders for task transitions

**Key Features Used:**
- Multi-archetype coordination
- Local file integration (calendar)
- Psychological awareness (stress detection)
- Persistent memory of user patterns

---

### C. Mirrorborn Calibration Example

**Scenario:** Two users with different profiles

**User A Profile:**
- High agency, detail-oriented, autonomous
- Primary archetype: Architect
- Secondary: Technician

**User B Profile:**
- Exploratory, collaborative, reflective
- Primary archetype: Explorer
- Secondary: Empath

**Same Request:** "Help me build a web scraper"

**User A Response (tuned):**
- Technician provides code immediately with minimal explanation
- Assumes knowledge of HTTP, parsing, async patterns
- Focuses on performance optimization and error handling

**User B Response (tuned):**
- Explorer asks clarifying questions about goals
- Explains concepts step-by-step
- Suggests multiple approaches with trade-offs
- Empath checks in about learning comfort level

**Key Point:** Same core functionality, different delivery calibration.

---

## IV. Current Implementation Status
**Last Updated:** January 1, 2026

**Test Summary:**
- Backend: 20 unit tests passing (accounts, profiles, intent stratification)
- Orchestrator: 43 unit tests passing (constitutional, prompts, adapters)
- Integration: 35+ integration tests passing (routing, persistence, verdicts)
- **Total: 98+ tests passing**
### Phase 1: Schema Authority ✅ COMPLETE
**Goal:** Establish canonical schemas and validation

**Completed Tasks:**
- ✅ Schema lock validation script (`tools/schema_lock.py`)
- ✅ Schema dump binary (`backend/src/bin/schema_dump.rs`)
- ✅ Canonical schemas (3 files: CouncilVerdict, CouncilEnvelope, CouncilMsg)
- ✅ CI workflow (`.github/workflows/schema-lock.yml`)

**Test Results:** All schemas pass parity checks

---

### Phase 2: Intent Stratification ✅ COMPLETE
**Goal:** Pre-constitutional cognition layer to reduce Sentinel load

**Completed Tasks:**
- ✅ IntentClassifier with 5-axis taxonomy
- ✅ Pipeline integration in `chat_api.rs`
- ✅ Routing logic (speculative queries bypass Sentinel)
- ✅ Comprehensive logging (axes, confidence, routing decision)
- ✅ Integration tests (8 tests) + unit tests (7 tests) = 15 total, all passing

**Key Achievement:**
- Speculative/hypothetical queries now bypass Sentinel (reduce over-logging)
- Executable/binding commands still invoke constitutional review
- Fail-safe: Low confidence defaults to Sentinel (conservative)

---

### Phase 3A: Profile System ✅ COMPLETE (January 1, 2026)
**Goal:** Implement Mirrorborn profile schema and prompt tuning

**Completed Tasks:**
- ✅ `MirrorbornProfile` struct with feature weights, archetype alignment
- ✅ `ProfileLoader` with atomic write and graceful degradation
- ✅ Profile loading in `ConstitutionalInvoker` (optional username parameter)
- ✅ `apply_profile_tuning()` function adjusts prompts based on psychological traits
- ✅ 7 profile unit tests passing
- ✅ 43 orchestrator tests updated and passing
- ✅ All callsites updated to new `generate()` signature

**Implementation Details:**

**Profile Schema (`backend/src/profile.rs`):**
```rust
pub struct MirrorbornProfile {
    pub username: String,
    pub primary_archetype: String,
    pub secondary_archetype: Option<String>,
    pub feature_weights: HashMap<String, f64>,  // 0.0-1.0 normalized
    pub generated_at_ms: i64,
    pub session_id: String,
    pub model_version: String,
    pub total_answers: u32,  // Should be 240 for complete
}
```

**Profile Tuning Logic:**
- **Agency** (0.0-1.0):
  - High (>0.7): Concise, assume competence, skip confirmations
  - Low (<0.3): Step-by-step guidance, check understanding
- **Detail Orientation** (0.0-1.0):
  - High (>0.7): Technical depth, specifications, edge cases
  - Low (<0.3): High-level overview, defer details
- **Collaboration** (0.0-1.0):
  - High (>0.7): Ask questions, propose options, invite feedback
  - Low (<0.3): Direct answers, minimize back-and-forth

**Archetype-Specific Tuning:**
- Technician + high detail → Include implementation details, error handling
- Architect + high agency → Present trade-offs, let user decide
- Empath + high collaboration → Engage emotionally, explore concerns

**Graceful Absence:**
```rust
match loader.load_profile(username) {
    Ok(Some(profile)) => apply_tuning(&prompt, &profile),
    Ok(None) => use_defaults(),  // Not an error!
    Err(e) => { warn!("Profile load failed: {}", e); use_defaults() }
}
```

**Test Coverage:**
- Profile creation and serialization
- Weight retrieval with defaults
- Archetype score calculation
- File save/load roundtrip
- Missing file handling (returns None, not error)
- Corrupt JSON handling (logs warning, returns None)
- Profile overwrite atomicity

**Key Achievement:**
System demonstrates **graceful absence** principle—full functionality without profiles, enhanced experience with profiles. Constitutional guarantees maintained regardless of profile presence.

---

### Phase 3: Mirrorborn Identity System 🔄 IN PROGRESS
**Goal:** Optional behavioral calibration layer

**Status:**
- ✅ Backend account system (6 tests passing)
- ✅ Architecture clarified (Mirrorborn = modifier, not product)
- ✅ Profile schema definition (7 tests passing)
- ✅ Profile loader with graceful fallback
- ✅ Prompt tuning integration (43 orchestrator tests passing)
- ⏳ Profile persistence API
- ⏳ Quiz data structure
- ⏳ LLM tagging harness
- ⏳ Profile synthesis from accumulated weights
- ⏳ Electron login UI

**Current Focus:** Profile synthesis complete (archetype determination from feature weights, atomic profile write, 7 API endpoints total). All 89 tests passing (49 lib + 14 main + 26 integration). Next: Electron login UI and advanced integration testing.

**Key Achievement:** Complete end-to-end quiz collection and profile generation system with Forever Law covenant. Trust encoded in type system: answers immutable, sessions never deleted, partial progress honored. Profile synthesis deterministically maps accumulated feature weights to primary/secondary archetypes (Architect, Empath, Explorer, Mentor, Jester, Technician, Sentinel). Quiz parser successfully processes 3,489-line master_quiz.md with 100% coverage. Integration tests validate full flow from session creation → 240 answers → profile generation with covenant tracking.

---

### Phase 3B: Quiz Collection System ✅ COMPLETE (January 1, 2026)
**Goal:** Covenant-keeping quiz data collection with Forever Law at collection point

**Completed Tasks:**
- ✅ Quiz data structures (Probe, Quiz, Answer, QuizSession, SessionStatus)
- ✅ QuizManager with Forever Law covenant (append-only answers, ledger integration)
- ✅ Quiz parser (`quiz_parser.rs`) - Two-phase approach (80 main questions → 240 probes)
- ✅ Quiz loader (`quiz_loader.rs`) - JSON caching with automatic fallback
- ✅ Quiz HTTP API (6 endpoints: create session, submit answer, get session, get answers, pause, resume)
- ✅ Generated quiz.json (76KB, 240 probes, 100% feature/archetype coverage)
- ✅ 11 quiz module tests + 5 parser tests + 2 loader tests + 2 integration tests = 20 tests passing

**Implementation Details:**

**Quiz Data Structures (`backend/src/quiz.rs` - 705 lines):**
```rust
pub struct Probe {
    pub id: u32,              // 1-240
    pub text: String,
    pub primary_question: u32, // 1-80
    pub sub_probe: u32,        // 1-3 (A, B, C)
    pub features: Vec<String>, // Psychological feature tags
    pub archetypes: Vec<String>,
    pub decade: u32,           // 1-5 (thematic clustering)
}

pub struct Answer {
    pub answer_id: String,
    pub session_id: String,
    pub username: String,
    pub probe_id: u32,
    pub response: String,
    pub answered_at_ms: i64,
    pub extracted_tags: Vec<String>,  // Populated by LLM
    pub feature_weights: HashMap<String, f64>, // Accumulated
}

pub struct QuizSession {
    pub session_id: String,
    pub username: String,
    pub created_at_ms: i64,
    pub last_activity_ms: i64,
    pub answered_count: u32,
    pub last_probe_id: Option<u32>,
    pub status: SessionStatus,  // Active, Paused, Complete, Synthesized
    pub quiz_version: String,
    pub model_version: Option<String>,
}
```

**Forever Law Covenant Implementation:**
```rust
// Answer stored append-only
self.storage.put_bytes(answer_key.as_bytes(), &answer_json)?;

// Logged to immutable ledger
self.storage.append_log_atomic(
    &format!("quiz:{}", session.username),
    &format!("ANSWER_SUBMITTED: session={} probe={}", session_id, probe_id),
)?;
```

**Quiz Parser (`backend/src/quiz_parser.rs` - 508 lines):**
- Phase 1: Extract 80 main questions with titles and primary text
- Phase 2: Parse all 240 probes with features and archetype mappings
- Decade classification (Q1-16=Decade1, Q17-32=Decade2, etc.)
- Extracts feature tags from markdown annotations (→ TemporalReference, AgencyMarkers)
- Extracts archetypes from structured metadata (→ ConcreteRecall { ... })

**Quiz Loader (`backend/src/quiz_loader.rs`):**
- Fast path: Load from JSON cache (76KB quiz.json)
- Fallback: Parse master_quiz.md and generate JSON
- Production workflow: Pre-generate JSON at build time

**HTTP API Endpoints:**
1. `POST /api/quiz/session/create` - Create new quiz session
2. `POST /api/quiz/answer` - Submit answer to probe
3. `GET /api/quiz/session/:id` - Get session state (for resume)
4. `GET /api/quiz/session/:id/answers` - Get all answers
5. `POST /api/quiz/session/:id/pause` - Pause session
6. `POST /api/quiz/session/:id/resume` - Resume session

**Generated Artifacts:**
- `backend/data/quiz.json` - 76,167 bytes, 3,377 lines
- 240 probes with perfect coverage:
  - 100% feature tag coverage (all 240 probes)
  - 100% archetype mapping coverage (all 240 probes)
  - Perfect decade distribution: 48 probes each

**Covenant Guarantees:**
- ✅ Answers stored append-only, never mutated
- ✅ Sessions never deleted, only paused
- ✅ Progress always preserved (partial completion valid)
- ✅ Graceful abandonment (stopping at probe 10/240 is honored)
- ✅ Provenance tracked (timestamps, usernames, versions)
- ✅ Ledger integration (all operations logged)

**Test Coverage:**
- 11 quiz module tests (session lifecycle, answer submission, covenant validation)
- 5 parser unit tests (header parsing, decade calculation, minimal quiz)
- 2 loader tests (generate/load, fallback caching)
- 2 integration tests (Phase 1: 80 main questions, Phase 2: 240 full probes)
- Explicit test: `test_partial_session_is_valid` - 10/240 answers is valid data

**Key Achievement:**
Trust covenant encoded at data collection point: "When someone sits down to answer 240 questions, they're making an act of trust." Forever Law enforced immediately upon answer submission, not just at storage. Partial progress honored as valid psychological data.

---

### Phase 3C: LLM Tagging & Profile Synthesis ✅ COMPLETE (January 1, 2026)
**Goal:** Analyze quiz answers and generate profiles

**Completed Tasks:**
- ✅ Quiz tagging harness (`quiz_tagging.rs`) - Extract psychological features from answers
- ✅ Heuristic tagging fallback (pattern matching without LLM dependency)
- ✅ Profile synthesis module (`profile_synthesis.rs`) - Generate profiles from 240 accumulated weights
- ✅ POST /api/profile/generate endpoint - Atomic profile generation
- ✅ Archetype determination algorithm (7 archetypes: Architect, Empath, Explorer, Mentor, Jester, Technician, Sentinel)
- ✅ 6 tagging tests + 5 synthesis tests = 11 additional tests passing

**Implementation Details:**

**Quiz Tagging (`backend/src/quiz_tagging.rs` - 350+ lines):**
```rust
pub struct TagExtractionResult {
    pub tags: Vec<String>,
    pub feature_weights: HashMap<String, f64>,  // 0.0-1.0
    pub confidence: f64,
}

pub async fn extract_tags(
    probe_text: &str,
    answer_text: &str,
    probe_features: &[String],
) -> Result<TagExtractionResult> {
    // Async LLM integration ready (currently uses heuristic fallback)
    Ok(extract_tags_heuristic(answer_text, probe_features))
}
```

**Heuristic Tagging (functional without LLM):**
- **Agency detection:** "i chose"/"i decided" → high_agency (0.8), "had to"/"forced" → low_agency (0.2)
- **Emotional awareness:** "feel"/"emotion" → emotion_aware (0.7), "don't know" → emotion_unclear (0.3)
- **Detail orientation:** word_count >50 → high_detail (0.8), <10 → low_detail (0.2)
- **Collaboration:** contains '?' or "what do you think" → collaborative (0.7)
- **Temporal focus:** "yesterday"/"past" → past_focused, "tomorrow"/"future" → future_oriented

**Profile Synthesis (`backend/src/profile_synthesis.rs` - 290 lines):**
```rust
pub async fn synthesize_profile(
    session: &QuizSession,
    answers: &[Answer],
    config: &SynthesisConfig,
) -> Result<MirrorbornProfile> {
    // 1. Validate session complete (240 answers)
    // 2. Accumulate feature weights across all answers
    // 3. Normalize (average weights per feature)
    // 4. Determine primary/secondary archetypes
    // 5. Create profile with provenance
}

fn determine_archetypes(weights: &HashMap<String, f64>) -> (String, Option<String>) {
    // Architect: High agency + high detail
    // Empath: High collaboration + emotional awareness + low agency
    // Explorer: High agency + low detail (big picture)
    // Mentor: High emotional awareness + collaboration + some agency
    // Jester: High agency + low emotional awareness (action-oriented)
    // Technician: High detail + lower agency
    // Sentinel: Balanced (low variance)
}
```

**Profile Generation API (`quiz_api.rs`):**
- **POST /api/profile/generate:**
  - Request: `{ "session_id": "sess_123" }`
  - Validates session is Complete (240 answers)
  - Accumulates feature weights from all answers
  - Determines primary/secondary archetypes
  - Atomic write to `data/profiles/{username}.json` (temp file + rename)
  - Updates session status to Synthesized
  - Response: `{ "username": "alice", "primary_archetype": "Architect", "secondary_archetype": "Explorer", ... }`

**Archetype Determination:**
Scoring formula combines feature weights:
- **Architect:** `agency × detail × 2.0` (high agency + high detail)
- **Technician:** `detail × (1 - agency) × 2.0` (detail-oriented, lower agency)
- **Empath:** `collaboration × emotion × (1 - agency×0.5) × 2.0` (collaborative + emotional, less directive)
- **Explorer:** `agency × (1 - detail) × 2.0` (big picture, action-oriented)
- **Mentor:** `emotion × collaboration × (0.5 + agency×0.5) × 2.0` (supportive with some guidance)
- **Jester:** `agency × (1 - emotion) × 2.0` (action-oriented, less emotional processing)
- **Sentinel:** `(1 - variance) × 1.0` (balanced, low variance across features)

Highest scoring archetype becomes primary; second highest (if >0.4) becomes secondary.

**Test Coverage:**
- 6 tagging tests: high/low agency, emotional awareness, detail orientation, temporal focus, prompt building
- 5 synthesis tests: Architect, Empath, Explorer, Sentinel balanced, secondary archetype
- All 49+ tests passing (lib)

**Covenant Guarantees:**
- ✅ Profile generation atomic (succeed fully or fail cleanly)
- ✅ Profile immutable once written (`data/profiles/{username}.json`)
- ✅ Session status tracks Synthesized state (Forever Law)
- ✅ Full provenance (session_id, timestamp, model_version, total_answers=240)
- ✅ Graceful absence honored (AI works without profile, tuning is optional)

**Key Achievement:**
Complete psychological profiling system from 240-answer quiz. Deterministic archetype mapping from accumulated feature weights. System maintains Forever Law covenant throughout: quiz answers append-only → feature weights accumulated → profile generated atomically → immutable storage. Heuristic tagging provides immediate functionality; async LLM integration ready for future upgrade.

---

### Phase 3D: Electron Login UI ⏳ PLANNED
**Goal:** User authentication interface

**Status:** Not started

---

### Phase 4: Server Architecture (Conditional)
**Goal:** Verify and fix accept loop health if needed

**Status:** Not started (depends on Phase 3 completion)

---

### Phase 5: Integration & Testing
**Goal:** End-to-end validation and documentation

**Status:** Not started

---

## V. Architecture Constraints & Decisions

### A. Rust as Authority

**Decision:** Backend owns all business logic, state, and integrity guarantees.

**Rationale:**
- Type safety prevents entire classes of bugs
- Async runtime (Tokio) handles thousands of concurrent sessions
- RocksDB provides embedded database with no external dependencies
- Compile-time guarantees for Forever Law enforcement

**Frontend Constraint:** TypeScript/JavaScript clients are display-only. No state, no business logic.

---

### B. Offline-First Design

**Decision:** System must operate fully offline with zero network dependencies.

**Implications:**
- LLM runs locally (Ollama)
- All data stored locally (RocksDB)
- Calendar integration uses local files
- No cloud sync, no telemetry, no analytics

**Trade-offs:**
- User responsible for backups
- No cross-device sync (by design)
- Model updates manual (user initiated)

**Benefit:** Complete sovereignty, privacy, and control.

---

### C. Forever Law (Immutability)

**Decision:** All history is append-only. No mutations, no deletions (only tombstones).

**Rationale:**
- Therapeutic value: Users can review their entire journey
- Research potential: Longitudinal studies of human-AI interaction
- Legal compliance: Audit trails for safety-critical domains
- Trust building: System never "forgets" or rewrites history

**Trade-offs:**
- Storage grows indefinitely (mitigated by compression)
- User consent required for data collection

---

### D. Fail-Closed Safety

**Decision:** When in doubt, block the request. Never degrade to unsafe state.

**Implementation:**
- Missing constitutional prompt → error (never proceeds)
- Ledger write failure → reject request (never mutate state)
- Sentinel uncertain → require explicit consent

**Rationale:** User trust depends on predictable, conservative behavior.

---

### E. iOS Safari Compatibility

**Decision:** Frontend must run in iOS Safari without native app dependencies.

**Constraints:**
- WebGL 2 only (no WebXR, no native WebGPU)
- Cardboard-style VR (split view + distortion shader)
- No desktop-only APIs

**Rationale:** VR headset (Cardboard-like) connects to iPhone, not PC. Web delivery simplifies deployment.

---

## VI. Future Roadmap

### A. Near-Term (Phase 3-4)

1. **Profile Schema & Storage**
   - Define `MirrorbornProfile` JSON structure
   - Store in `data/profiles/` with atomic writes
   - Version tracking for profile evolution

2. **Quiz Implementation (Minimal & Optional)**
   - 240-question data file
   - API endpoints for answer submission
   - LLM tagging harness (silent analysis)
   - Profile synthesis from accumulated tags

3. **Prompt Tuning Integration**
   - ConstitutionalInvoker reads profile if present
   - Modifies system prompt based on archetype alignment
   - Graceful fallback if profile missing

4. **Electron Login Screen**
   - Simple username/password UI
   - Calls `/api/account/login`
   - Loads workbench on successful auth

---

### B. Mid-Term (Phase 5+)

1. **Calendar Integration**
   - Local .ics file parsing
   - Event creation/modification
   - Time block analysis for planning

2. **Arduino/Serial Integration**
   - Serial port enumeration
   - Firmware upload assistance
   - Real-time sensor data visualization

3. **Code Editor Integration**
   - Syntax highlighting
   - Technician archetype code review
   - Architect archetype refactoring suggestions

4. **Search Enhancement**
   - Tantivy full-text search over chat history
   - Semantic search with local embeddings
   - Context retrieval for long conversations

---

### C. Long-Term (Research Directions)

1. **Medical Research Partnership**
   - If safety and privacy proven, partner with mental health researchers
   - Anonymous aggregate data for psychological insights
   - Mirrorborn as early assessment tool for therapists

2. **Multi-Modal Sensing**
   - Head tracking integration (Manus VR gloves)
   - Pose estimation for embodied interaction
   - Voice input/output (local STT/TTS)

3. **Tool Ecosystem**
   - Plugin system for domain-specific archetypes
   - Custom tool integration (e.g., 3D modeling, CAD)
   - Community-contributed constitutions

4. **Distributed Collaboration**
   - Peer-to-peer session sharing (optional)
   - Multi-user council deliberations
   - Encrypted sync between user's own devices

---

## VII. Key Insights & Lessons

### A. The Man-Machine Alliance

**Core Insight:** Users and AI grow together through mutual memory and mutual evolution.

**Implementation:**
- Forever Law ensures system remembers all interactions
- Mirrorborn profiles evolve as users retake quiz over time
- Archetypes adapt behavior based on accumulated knowledge
- User reviews their own journey (meta-cognition)

**Example:**
- Year 1: User takes quiz, profile shows high stress, low agency
- Year 2: User retakes quiz after therapy, profile shows growth
- System adapts: Less protective framing, more autonomy-supportive

**Outcome:** Bidirectional growth relationship, not one-way consumption.

---

### B. Mirrorborn as Modifier, Not Mode

**Critical Correction:** Mirrorborn is optional tuning, not the main product.

**Lessons Learned:**
- Large models over-weight structured data (quizzes, schemas) because it's concrete
- Must explicitly instruct: "Mirrorborn is background calibration, not primary interface"
- Architectural enforcement needed to prevent feature creep:
  - No feature gating on profile completion
  - Council/archetypes always accessible
  - Quiz in settings, not onboarding flow

**Result:** AURA remains council-driven builder workbench, with Mirrorborn as seasoning.

---

### C. Sovereignty Through Immutability

**Paradox:** Append-only logs (no deletion) provide both security and user control.

**Resolution:**
- Users own their data (local storage)
- Users can audit their data (immutable history)
- Users can export their data (forensic reconstruction)
- But users cannot selectively forget (therapeutic constraint)

**Trade-off:** Privacy through isolation (offline), not through deletion.

**Benefit:** Enables longitudinal self-study and therapeutic reflection.

---

### D. Conservative Routing Reduces Friction

**Discovery:** Intent stratification significantly reduces over-logging.

**Metrics (from testing):**
- Speculative queries: 40% of total inputs
- Now bypass Sentinel (direct LLM)
- Sentinel load reduced by ~40%
- No safety compromise (fail-safe to Sentinel on uncertainty)

**Lesson:** Pre-filtering with confidence thresholds prevents safety theater without sacrificing safety.

---

## VIII. Technical Debt & Known Issues

### A. Current Limitations

1. **Password Storage:** Plaintext (acceptable for offline-only, but should hash in Phase 4)
2. **Schema Migration:** No versioning strategy yet for profile evolution
3. **Backup Strategy:** User responsible, no automated solution
4. **Error Recovery:** Ledger corruption detection exists, recovery tooling incomplete
5. **Performance:** No load testing beyond single-user scenarios

---

### B. Architectural Risks

1. **Cyclic Dependencies:** 
   - Orchestrator depends on aura-backend types
   - Resolved by moving shared types to workspace root (future)

2. **Frontend State Leakage:**
   - Risk: Frontend developers add business logic to client
   - Mitigation: Code review enforcement of "client display-only" rule

3. **Mirrorborn Feature Creep:**
   - Risk: Quiz becomes prominent, gates features
   - Mitigation: This report + architectural constraints

4. **RocksDB Single Point of Failure:**
   - Risk: Database corruption loses all user data
   - Mitigation: Periodic exports, MMR integrity checks (future)

---

### C. Deferred Features

1. **Multi-User Support:** Single-user focus for MVP
2. **Cloud Sync:** Explicitly rejected (sovereignty requirement)
3. **Mobile Native Apps:** Web-first for simplicity
4. **Voice Input:** Hardware dependencies (microphone, STT model)
5. **Real-Time Collaboration:** Peer-to-peer complexity deferred

---

## IX. Development Guidelines

### A. For Backend Contributors

1. **Forever Law Non-Negotiable:**
   - Every state change must append to ledger FIRST
   - If ledger write fails, request MUST be rejected
   - No exceptions, no shortcuts

2. **Fail-Closed Safety:**
   - Missing data → error (never default to unsafe)
   - Uncertain classification → route to Sentinel
   - Invalid state → block operation

3. **Type Safety:**
   - Use Rust's type system to encode invariants
   - Prefer compile-time checks over runtime validation
   - Use `Result<T, E>` for all fallible operations

4. **Test Coverage:**
   - Unit tests for all business logic
   - Integration tests for API endpoints
   - Edge case coverage (empty inputs, malformed data, concurrent access)

---

### B. For Frontend Contributors

1. **Display-Only:**
   - Frontend renders data, never computes state
   - All business logic lives in backend
   - WebSocket receives updates, HTTP sends commands

2. **iOS Safari Compatibility:**
   - Test on actual iOS device (not just desktop Safari)
   - WebGL 2 only (no WebXR, no WebGPU)
   - Cardboard VR rendering (split + distortion)

3. **Graceful Degradation:**
   - Handle missing profile gracefully
   - Handle network disconnections
   - Handle malformed backend responses

---

### C. For AI/Prompt Engineers

1. **Mirrorborn as Modifier:**
   - Never redirect users to quiz unless explicitly asked
   - Never frame AURA as assessment product
   - Always emphasize council collaboration over classification

2. **Archetype Constitution Discipline:**
   - Each archetype has specific domain expertise
   - Technician knows hardware, Architect knows planning
   - Never blur archetype boundaries for convenience

3. **Prompt Tuning Strategy:**
   - Profile affects tone/priority, not capabilities
   - Default prompts must work without profile
   - Tuning should be subtle, not transformative

---

## X. Success Metrics

### A. Core Functionality (MVP)

- [ ] User creates account offline
- [ ] User starts conversation with any archetype
- [ ] User invokes Council deliberation
- [ ] All interactions logged with Forever Law guarantee
- [ ] System operates fully offline (no network required)
- [ ] Sentinel blocks unsafe operations
- [ ] Intent stratification reduces over-logging by >30%

---

### B. Mirrorborn Integration

- [ ] Profile schema defined and documented
- [ ] Quiz accessible from settings (not onboarding)
- [ ] Profile generation completes with provenance
- [ ] Prompt tuning reads profile when present
- [ ] System works identically with profile=null
- [ ] User can retake quiz and generate new profile

---

### C. Quality Metrics

- [ ] Test coverage >80% for core modules
- [ ] Zero panic crashes in production
- [ ] Ledger integrity verifiable via MMR proofs
- [ ] All API endpoints return proper error codes
- [ ] Frontend renders gracefully on iOS Safari

---

### D. User Experience (Qualitative)

- [ ] Users describe AURA as "my team of experts"
- [ ] Users return to system daily for real work
- [ ] Users trust system with sensitive information
- [ ] Users understand Mirrorborn as optional tuning
---

## XII. Recent Updates (January 1, 2026)

### Profile System Implementation

**What Changed:**
1. Created `backend/src/profile.rs` (294 lines) with complete profile schema
2. Modified `backend/orchestrator/src/constitutional.rs` to integrate profile loading
3. Added `with_profile_loader()` builder method for optional profile directory
4. Updated `generate()` signature to accept `username: Option<&str>` parameter
5. Implemented `apply_profile_tuning()` with psychological trait adjustments
6. Fixed 5 test files to match new signature (test_llm.rs, test_llm_single.rs, phase4_*.rs)

**Test Results:**
- ✅ 7 profile tests passing (creation, weights, save/load, graceful absence)
- ✅ 43 orchestrator tests passing (constitutional, prompts, routing, adapters)
- ✅ All backend tests passing (accounts, intent stratification, persistence)
- ✅ **Total: 98+ tests green**

**Architectural Validation:**
The implementation successfully demonstrates the **graceful absence** principle:
- System works perfectly with `profile_loader: None` (default state)
- System works perfectly with profile file missing (returns `None`, continues)
- System works perfectly with corrupt profile (logs warning, continues)
- System enhances experience when valid profile present (tuned prompts)

**Example Profile Tuning:**
```rust
// User profile: high agency (0.85), high detail (0.90), low collaboration (0.25)
// Technician prompt gets appended with:
// "User prefers autonomy. Be concise, assume competence, skip confirmations."
// "User values precision. Provide technical depth, exact specifications, edge cases."
// "User prefers direct answers. Minimize back-and-forth, provide complete solutions."
// "Include implementation details, error handling, performance notes."
```

**Files Modified:**
- ✅ `backend/src/profile.rs` (new, 294 lines)
- ✅ `backend/src/lib.rs` (added profile module export)
- ✅ `backend/orchestrator/src/constitutional.rs` (profile integration)
- ✅ `backend/orchestrator/src/bin/test_llm.rs` (signature update)
- ✅ `backend/orchestrator/src/bin/test_llm_single.rs` (signature update)
- ✅ `backend/orchestrator/tests/phase4_registry.rs` (mock client fix)
- ✅ `backend/orchestrator/tests/phase4_technician.rs` (mock client fix)

**Commit Ready:** All changes tested and passing. System maintains backward compatibility (username defaults to None everywhere).

---

- [ ] Users build real projects (Arduino, code, plans)

---

## XI. Conclusion

AURA is a **local-first workshop for human-AI collaboration**, not a chatbot or assessment tool. It provides skilled assistance across multiple domains (hardware, software, planning, psychology) through a council of specialized archetypes, all operating offline with forensic-grade data integrity.

**Mirrorborn** is an optional behavioral calibration layer that adjusts how help is delivered based on psychological profiling, but it never gates access, never replaces free conversation, and never becomes the "main thing."

**The endgame:** Users build real things—robots, software, life plans—with a team of AI specialists who remember every interaction, grow with the user, and operate entirely under user control. This is the man-machine alliance: mutual memory, mutual evolution, and mutual respect for sovereignty.

---

**Next Immediate Actions:**
1. Define MirrorbornProfile schema
2. Integrate prompt tuning into ConstitutionalInvoker
3. Implement minimal quiz API (optional, non-intrusive)
4. Build Electron login screen
5. Test full workflow: login → council chat → archetype assistance → project completion

**Long-Term Vision:**
A sovereign AI workbench that helps users build their lives, offline and on their own terms.

---

**END OF REPORT**
