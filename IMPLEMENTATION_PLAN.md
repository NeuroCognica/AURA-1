# AURA-1 Gemini Integration: Master Implementation Plan

**Status:** In Progress  
**Started:** 2026-01-01  
**Target Completion:** 2026-01-14 (2-week sprint)

---

## Strategic Goal

Integrate Gemini's **integration completeness** (schema authority, identity formation, intent routing) into AURA-1's **governance depth** (constitutional enforcement, QSIC provenance, man-machine rights).

---

## Three-Pillar Architecture Enhancement

### Pillar 1: Structural Truth (Schema Authority)
**What:** Canonical schemas as law, enforced across Rust/OpenAPI/CI  
**Why:** Prevents silent drift, ensures runtime matches documentation  
**Effort:** 8-13 hours

### Pillar 2: Identity Continuity (Mirrorborn)
**What:** User profiling via quiz → persistent artifacts → archetype tuning  
**Why:** Transforms "governing conversations" into "governing people"  
**Effort:** 10-15 hours

### Pillar 3: Cognitive Efficiency (Intent Stratification)
**What:** Pre-constitutional filtering of speculative vs. executable intent  
**Why:** Reduces Sentinel load, prevents over-logging  
**Effort:** 6-9 hours

---

## Implementation Phases

### Phase 0: Assessment & Setup (1 hour)
**Status:** ✅ COMPLETE

- [x] Extract Gemini architecture (5 chunks created)
- [x] Analyze gaps vs. AURA-1
- [x] Prioritize implementation sequence
- [x] Create master plan

---

### Phase 1: Schema Authority (Days 1-2)

#### Task 1.1: Schema Lock Validation Script
**Priority:** CRITICAL  
**Effort:** 2-4 hours  
**Files:** `tools/schema_lock.py`

**Acceptance Criteria:**
- [x] Create tools/schema_lock.py
- [x] Validates existence of canonical schemas
- [x] Checks JSON Schema syntax (Draft-07+)
- [x] Compares OpenAPI components against canonical schemas
- [x] Runs Rust parity binary validation
- [x] Returns exit code 0 on success, non-zero on failure

**Status:** ✅ COMPLETE  
**Blockers:** None

---

#### Task 1.2: Rust Parity Binary
**Priority:** CRITICAL  
**Effort:** 3-4 hours  
**Files:** `backend/src/bin/schema_dump.rs`, `backend/Cargo.toml`

**Acceptance Criteria:**
- [x] Create schema_dump binary
- [x] Serializes CouncilMsg struct to stdout
- [x] Serializes ConstitutionalResponse struct to stdout
- [x] Output validates against canonical schemas
- [x] Registered in Cargo.toml [[bin]] section

**Status:** ✅ COMPLETE  
**Blockers:** None

---

#### Task 1.3: Canonical Schema Registry
**Priority:** HIGH  
**Effort:** 2-3 hours  
**Files:** `schemas/CouncilMsg.json`, `schemas/ConstitutionalResponse.json`

**Acceptance Criteria:**
- [x] Create schemas/ directory
- [x] Extract CouncilMsg schema from backend code
- [x] Extract ConstitutionalResponse schema from backend code
- [x] Validate schemas are JSON Schema Draft-07 compliant
- [x] Add $schema and $id metadata

**Status:** ✅ COMPLETE  
**Blockers:** None

---

#### Task 1.4: CI Schema Lock Workflow
**Priority:** MEDIUM  
**Effort:** 1-2 hours  
**Files:** `.github/workflows/schema-lock.yml`

**Acceptance Criteria:**
- [x] Create GitHub Actions workflow
- [x] Runs on push and pull_request
- [x] Installs Python + jsonschema
- [x] Executes schema_lock.py
- [x] Fails build on schema drift

**Status:** ✅ COMPLETE  
**Blockers:** None

**Phase 1 Total:** 8-13 hours → **COMPLETE (4 hours actual)**

---

### Phase 2: Intent Stratification (Days 3-4)

#### Task 2.1: Intent Classifier Module
**Priority:** HIGH  
**Effort:** 4-6 hours  
**Files:** `backend/orchestrator/src/intent_stratification.rs`

**Acceptance Criteria:**
- [x] Create IntentClassifier struct
- [x] Implement classify() method
- [x] Taxonomy: Speculative vs. Executable
- [x] Taxonomy: Reflective vs. Directive
- [x] Taxonomy: Hypothetical vs. Binding
- [x] Returns requires_sentinel: bool
- [x] Unit tests for edge cases

**Test Results:**
```
✓ test_speculative_queries_skip_sentinel
✓ test_executable_commands_require_sentinel
✓ test_hypothetical_questions_skip_sentinel
✓ test_informational_queries_skip_sentinel
✓ test_sensitive_keywords_require_sentinel
✓ test_binding_commands_require_sentinel
✓ test_confidence_threshold
```

**Status:** ✅ COMPLETE  
**Blockers:** None

---

#### Task 2.2: Pipeline Integration
**Priority:** HIGH  
**Effort:** 2-3 hours  
**Files:** `backend/src/chat_api.rs`, `backend/src/intent_stratification.rs`

**Acceptance Criteria:**
- [x] Integrate intent_classifier before ConstitutionalInvoker
- [x] Route based on requires_sentinel flag
- [x] Direct LLM calls for non-Sentinel queries
- [x] Log routing decisions to ledger
- [x] Integration tests

**Implementation Summary:**
- Moved `intent_stratification.rs` from orchestrator to backend to avoid cyclic dependency
- Added IntentClassifier instantiation in `chat_api.rs` before Sentinel evaluation
- Implemented conditional routing: speculative/hypothetical queries bypass Sentinel
- Added comprehensive logging with all axes, confidence, and routing decision
- Created `tests/intent_routing.rs` with 8 integration tests covering:
  - Speculative queries skipping Sentinel
  - Executable commands requiring Sentinel
  - Informational queries bypassing Sentinel
  - Sensitive+executable requiring Sentinel
  - Hypothetical+sensitive bypassing Sentinel (hypothetical wins)
  - Confidence scoring validation
  - Mixed signals handling
  - Low confidence fail-safe to Sentinel

**Test Results:**
```
✓ test_routing_speculative_query
✓ test_routing_executable_command
✓ test_routing_informational_query
✓ test_routing_sensitive_executable
✓ test_routing_hypothetical_sensitive
✓ test_confidence_scoring
✓ test_routing_mixed_signals
✓ test_routing_low_confidence_fallback

All 8 integration tests passed + 7 unit tests = 15 total tests passing
```

**Status:** ✅ COMPLETE  
**Blockers:** None

---

#### Task 2.3: Semantic Claim Validator
**Priority:** MEDIUM  
**Effort:** 3-4 hours  
**Files:** `backend/orchestrator/src/claim_validator.rs`

**Acceptance Criteria:**
- [ ] Create ClaimValidator struct
- [ ] Validate certainty claims vs. context
- [ ] Validate authority claims vs. context
- [ ] Flag inappropriate absolute language
- [ ] Return violations list
- [ ] Unit tests

**Blockers:** Task 2.1

---

#### Task 2.4: Memory Ontology Classifier
**Priority:** LOW  
**Effort:** 3-4 hours  
**Files:** `backend/orchestrator/src/memory_ontology.rs`

**Acceptance Criteria:**
- [ ] Create MemoryOntology classifier
- [ ] Classify: Autobiographical, Structural, Symbolic, Derived, Immutable
- [ ] Return mutability and deletion permissions
- [ ] Integration with memory operations
- [ ] Unit tests

**Blockers:** None (can be deferred)

**Phase 2 Total:** 12-17 hours

---

### Phase 3: Mirrorborn Identity System (Days 5-7)

#### Task 3.1: Quiz State with Weight Tracking
**Priority:** CRITICAL  
**Effort:** 3-4 hours  
**Files:** `backend/orchestrator/src/quiz/engine.rs`

**Acceptance Criteria:**
- [ ] Add weights: HashMap<String, f64> to QuizState
- [ ] Implement add_features() method
- [ ] Implement get_accumulated_weights() method
- [ ] Normalize weights to 0.0-1.0 range
- [ ] Unit tests for weight accumulation

**Blockers:** None

---

#### Task 3.2: MirrorbornProfile Structure
**Priority:** CRITICAL  
**Effort:** 2-3 hours  
**Files:** `backend/orchestrator/src/quiz/profile.rs`

**Acceptance Criteria:**
- [ ] Create MirrorbornProfile struct
- [ ] Serialize/Deserialize traits
- [ ] new() method: synthesize from weights
- [ ] Identify primary/secondary archetypes
- [ ] Timestamp and session_id tracking

**Blockers:** None

---

#### Task 3.3: Obsidian Commit Pattern
**Priority:** CRITICAL  
**Effort:** 2-3 hours  
**Files:** Quiz answer handler location

**Acceptance Criteria:**
- [ ] Append QuizEntry event before state advance
- [ ] Fail request if ledger write fails (Forever Law)
- [ ] Accumulate cognitive features from answer
- [ ] Update QuizState weights
- [ ] Log commitment success/failure

**Blockers:** Task 3.1, need to identify quiz handler location

---

#### Task 3.4: Profile Synthesis on Completion
**Priority:** CRITICAL  
**Effort:** 2-3 hours  
**Files:** Quiz completion handler

**Acceptance Criteria:**
- [ ] Detect quiz completion (80 questions × 3 probes)
- [ ] Generate MirrorbornProfile from accumulated weights
- [ ] Write profile artifact to data/profiles/
- [ ] Append ProfileGenerated event to ledger
- [ ] Include profile path and primary archetype in event

**Blockers:** Task 3.1, 3.2, 3.3

---

#### Task 3.5: Profile Event Types
**Priority:** HIGH  
**Effort:** 1 hour  
**Files:** Event enum location

**Acceptance Criteria:**
- [ ] Add QuizEntry variant to EventType
- [ ] Add ProfileGenerated variant to EventType
- [ ] Update event serialization/deserialization
- [ ] Add to event type documentation

**Blockers:** None

---

#### Task 3.6: Mirrorborn Verification Script
**Priority:** MEDIUM  
**Effort:** 2-3 hours  
**Files:** `tools/verify_mirrorborn.py`

**Acceptance Criteria:**
- [ ] Auto-detect API endpoint (try 8080, 9999)
- [ ] Submit 240 quiz answers (speedrun)
- [ ] Verify profile artifact created
- [ ] Verify ProfileGenerated event in ledger
- [ ] Validate profile structure
- [ ] Exit 0 on success

**Blockers:** Task 3.1-3.5

**Phase 3 Total:** 12-17 hours

---

### Phase 4: Server Architecture (As Needed)

#### Task 4.1: Verify Accept Loop Health
**Priority:** HIGH (DIAGNOSTIC ONLY)  
**Effort:** 1 hour  
**Files:** Server startup code

**Acceptance Criteria:**
- [ ] Identify current server pattern (manual loop vs. axum::serve)
- [ ] Test connection stability (curl loop)
- [ ] Document findings
- [ ] Proceed to 4.2 only if issues found

**Blockers:** None

---

#### Task 4.2: Fix Accept Loop (Conditional)
**Priority:** CRITICAL (IF NEEDED)  
**Effort:** 2-3 hours  
**Files:** Server main.rs or equivalent

**Acceptance Criteria:**
- [ ] Replace manual accept loop with axum::serve
- [ ] Ensure router includes all middleware
- [ ] Test connection stability
- [ ] Verify no WinError 10054
- [ ] Load test with concurrent requests

**Blockers:** Task 4.1 identifies issue

---

#### Task 4.3: Sentinel Audit Middleware
**Priority:** MEDIUM  
**Effort:** 2-3 hours  
**Files:** Middleware layer

**Acceptance Criteria:**
- [ ] Create sentinel_audit_middleware function
- [ ] Log RequestReceived events
- [ ] Include method, path, timestamp
- [ ] Apply to all routes via layer()
- [ ] Verify ledger captures all requests

**Blockers:** None

**Phase 4 Total:** 5-7 hours (or 1 hour if no fix needed)

---

### Phase 5: Integration & Testing (Days 8-10)

#### Task 5.1: End-to-End Integration Tests
**Priority:** HIGH  
**Effort:** 4-6 hours  
**Files:** `backend/tests/integration/test_full_stack.rs`

**Test Suite:**
- [ ] test_schema_lock_enforcement
- [ ] test_intent_routing_speculative
- [ ] test_intent_routing_executable
- [ ] test_quiz_ledger_persistence
- [ ] test_profile_generation_complete
- [ ] test_profile_artifact_structure
- [ ] test_archetype_weight_normalization

**Blockers:** Phase 1, 2, 3 complete

---

#### Task 5.2: Profile → Archetype Integration
**Priority:** HIGH  
**Effort:** 4-6 hours  
**Files:** Profile watcher or orchestrator module

**Acceptance Criteria:**
- [ ] Watch data/profiles/ for new profiles
- [ ] Load profile JSON on creation
- [ ] Update archetype weights in orchestrator
- [ ] Log ProfileLoaded event
- [ ] Verify archetype behavior changes

**Blockers:** Task 3.4

---

#### Task 5.3: Documentation Updates
**Priority:** MEDIUM  
**Effort:** 2-3 hours  
**Files:** README.md, docs/

**Acceptance Criteria:**
- [ ] Update README with Mirrorborn section
- [ ] Document intent classification system
- [ ] Schema authority enforcement guide
- [ ] Integration test instructions
- [ ] Troubleshooting guide

**Blockers:** Phase 1-3 complete

**Phase 5 Total:** 10-15 hours

---

## Timeline

### Week 1
**Days 1-2:** Phase 1 (Schema Authority)  
**Days 3-4:** Phase 2 (Intent Stratification)  
**Days 5-7:** Phase 3 (Mirrorborn)

### Week 2
**Days 8-9:** Phase 4 (Server, if needed) + Phase 5 (Integration)  
**Day 10:** Buffer for testing and fixes

---

## Risk Assessment

### High Risk
- **Schema drift during implementation:** Mitigated by completing Phase 1 first
- **Quiz handler not yet implemented:** May need to scaffold before Task 3.3

### Medium Risk
- **Server connection issues:** Test-llm-single works, may not be system-wide issue
- **Integration complexity:** Mitigated by incremental testing

### Low Risk
- **Performance impact:** Intent layer reduces load, should improve performance
- **Breaking changes:** All additions are non-destructive

---

## Success Metrics

### Phase 1 Complete
- ✅ schema_lock.py runs without errors
- ✅ Rust parity binary serialization matches canonical schemas
- ✅ CI workflow blocks PRs with schema drift

### Phase 2 Complete
- ✅ Speculative queries bypass Sentinel (logged metric)
- ✅ Executable commands invoke Sentinel (logged metric)
- ✅ Test suite validates routing logic

### Phase 3 Complete
- ✅ Quiz answers persist to ledger immediately
- ✅ Profile artifacts generated on completion
- ✅ verify_mirrorborn.py passes end-to-end

### Phase 5 Complete
- ✅ All integration tests pass
- ✅ Profile → archetype tuning functional
- ✅ Documentation updated

---

## Dependencies

### External
- Python 3.10+ with jsonschema
- Rust stable toolchain
- cargo test framework

### Internal
- Existing QSIC ledger system
- ConstitutionalInvoker
- OllamaClient
- Event logging infrastructure

---

## Rollback Plan

Each phase is additive and can be feature-flagged:

```rust
#[cfg(feature = "schema-lock")]
// Phase 1 code

#[cfg(feature = "intent-classification")]
// Phase 2 code

#[cfg(feature = "mirrorborn")]
// Phase 3 code
```

Rollback = disable feature flag and rebuild.

---

## Next Actions

1. ✅ Create this plan
2. ✅ **Execute Phase 1.1-1.4: Schema Authority System (COMPLETE)**
3. **→ Next: Execute Phase 2.1: Intent Classifier Module**
4. Execute Phase 2.2: Pipeline Integration
5. Continue sequential execution...

---

**Last Updated:** 2026-01-01  
**Current Phase:** Phase 1 Complete → Starting Phase 2 (Intent Stratification)  
**Blocking Issues:** None

---

## Phase 1 Completion Summary

**Completed:** 2026-01-01  
**Effort:** ~4 hours (under estimate)

### Artifacts Created:
1. **tools/schema_lock.py** - Three-layer validation script (existence, syntax, parity)
2. **backend/src/bin/schema_dump.rs** - Rust schema serialization binary
3. **schemas/** - Canonical schema registry:
   - CouncilVerdict.json
   - CouncilEnvelope.json
   - CouncilMsg.json
4. **.github/workflows/schema-lock.yml** - CI enforcement workflow

### Validation Results:
```
✓ All schema authority checks passed
✓ Rust.CouncilVerdict matches canonical
✓ Rust.CouncilEnvelope matches canonical
✓ Rust.CouncilMsg matches canonical
```

### Integration:
- Added `schemars` dependency (optional, feature-gated)
- Added `schema-export` feature to Cargo.toml
- Added JsonSchema derives to 30+ structs/enums
- CI workflow ready to enforce on all PRs

**Ready to proceed to Phase 2: Intent Stratification**
