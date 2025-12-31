# AURA / NeuroCognica — MASTER IMPLEMENTATION CHECKLIST

*(Constitution → Cognition → Control)*

---

## PHASE 0 — CONSTITUTIONAL FREEZE (COMPLETE)

> Goal: Lock authority, time, and refusal semantics permanently.

* [x] Typed `CouncilMsg` enum defined
* [x] `CouncilEnvelope` wrapper implemented
* [x] Deterministic serde tests added
* [x] Persist-first authority pipeline implemented
* [x] Session-scoped sequence allocation in storage
* [x] Atomic write (envelope + last_seq) guaranteed
* [x] `/ws/council` authoritative channel enforced
* [x] `/ws/ai` non-authoritative by design
* [x] Generation cancellation enforced server-side
* [x] Client token drop on authority violation
* [x] Replay protocol implemented (hello / ack / replay / replay_done)
* [x] Live stream blocked until replay completion
* [x] End-to-end WebSocket replay integration test passing
* [x] Tag created: `aura-1-authority-spine-v1`
* [x] Release drafted/published
* [x] Authority spine merged to `main`

**Invariant:** Authority is explicit, persisted, replayable, and non-negotiable.

---

## PHASE 1 — REGRESSION IMMUNITY (IMMEDIATE)

> Goal: Prevent constitutional decay.

### CI/CD Implementation


* [x] Create `.github/workflows/authority-spine-ci.yml`
	* [x] Configure workflow to trigger on push to `main` and all PRs
	* [x] Set up Rust toolchain installation (stable)
	* [x] Add `cmake` and `nasm` as build dependencies
	* [x] Configure cargo cache for faster builds
* [x] Add test matrix for feature combinations
	* [x] Test with `persistence` feature enabled
	* [x] Test with `persistence` feature disabled
	* [x] Test with `search` feature enabled
	* [x] Test with `search` feature disabled
* [ ] Configure test execution
	* [x] Run `cargo test --all-features`
	* [x] Run `cargo test --no-default-features`
	* [x] Explicitly run `ws_replay_integration` test
	* [x] Run `council_envelope_serde` deterministic tests
* [ ] Add build verification
	* [ ] Run `cargo build --release --all-features`
	* [ ] Run `cargo clippy -- -D warnings`
	* [ ] Run `cargo fmt -- --check`
* [ ] Configure failure conditions
	* [ ] Fail on any test failure
	* [ ] Fail on clippy warnings
	* [ ] Fail on formatting violations
	* [ ] Block PR merge on CI failure
* [ ] Add status badge to README.md
* [ ] Verify CI runs successfully on `main`
* [ ] Document CI as constitutional requirement in README
	* [ ] Add section: "Constitutional Guarantees"
	* [ ] Explain authority regression prevention
	* [ ] Link to CI workflow file

**Invariant:** No commit can weaken authority guarantees.

---

## PHASE 2 — OBSERVABILITY FOUNDATION

> Goal: Make authority *visible* without changing behavior.

### Metrics Infrastructure

* [ ] Add dependencies to `Cargo.toml`
	* [ ] Add `prometheus = "0.13"`
	* [ ] Add `lazy_static = "1.4"` for metric registration
* [ ] Create `backend/src/metrics.rs` module
	* [ ] Define `COUNCIL_ENVELOPES_PERSISTED` counter
	* [ ] Define `WS_COUNCIL_CONNECTIONS_ACTIVE` gauge
	* [ ] Define `WS_REPLAY_REQUESTS_TOTAL` counter
	* [ ] Define `SESS_COUNCIL_LAST_SEQ` gauge with `session_id` label
	* [ ] Export `register_metrics()` function
* [ ] Integrate metrics into `main.rs`
	* [ ] Call `register_metrics()` at startup
	* [ ] Add `/metrics` endpoint handler
	* [ ] Expose endpoint on same port as API
* [ ] Instrument `storage.rs`
	* [ ] Increment `COUNCIL_ENVELOPES_PERSISTED` after successful persist
	* [ ] Update `SESS_COUNCIL_LAST_SEQ` gauge after sequence allocation
* [ ] Instrument `broadcast.rs`
	* [ ] Increment `WS_COUNCIL_CONNECTIONS_ACTIVE` on connection
	* [ ] Decrement `WS_COUNCIL_CONNECTIONS_ACTIVE` on disconnect
	* [ ] Increment `WS_REPLAY_REQUESTS_TOTAL` on replay request
* [ ] Add metrics verification test
	* [ ] Create `backend/tests/metrics_integration.rs`
	* [ ] Start server, persist envelope, verify counter incremented
	* [ ] Connect WebSocket, verify gauge incremented
	* [ ] Disconnect, verify gauge decremented
* [ ] Verify metrics do not affect authority timing
	* [ ] Run `ws_replay_integration` test with metrics enabled
	* [ ] Confirm no timing regressions
	* [ ] Confirm replay order unchanged
* [ ] Document metrics in README
	* [ ] Add "Observability" section
	* [ ] List all exposed metrics
	* [ ] Provide example Prometheus scrape config

### Optional: Local Grafana Dashboard

* [ ] Create `docker-compose.yml` for local monitoring stack
	* [ ] Add Prometheus service
	* [ ] Add Grafana service
	* [ ] Configure Prometheus to scrape AURA-1 metrics
* [ ] Create Grafana dashboard JSON
	* [ ] Panel: Council envelopes persisted over time
	* [ ] Panel: Active WebSocket connections
	* [ ] Panel: Replay requests rate
	* [ ] Panel: Per-session last sequence number
* [ ] Document dashboard setup in `docs/observability.md`

**Invariant:** Observability must never influence authority.

---

## PHASE 3 — ARCHETYPE RUNTIME (SENTINEL FIRST)

> Goal: Prove cognition can safely sit on the authority spine.

### Repository Setup

* [ ] Create new repository: `NeuroCognica/aura-sentinel`
	* [ ] Initialize with Rust project: `cargo init --name aura-sentinel`
	* [ ] Add MIT license
	* [ ] Create README with archetype description
	* [ ] Add `.gitignore` for Rust projects

### Sentinel Archetype Configuration

* [ ] Create `config/sentinel.json`
	* [ ] Define archetype metadata: name, role, temperature
	* [ ] Set `temperature: 0.0` (deterministic)
	* [ ] Set `top_p: 0.1` (constrained)
	* [ ] Define forbidden domains: ["user_override", "authority_generation"]
	* [ ] Define allowed message types: ["observation", "analysis", "verdict"]
* [ ] Create config loader in `src/config.rs`
	* [ ] Define `SentinelConfig` struct
	* [ ] Implement JSON deserialization
	* [ ] Add validation for required fields
	* [ ] Add test for config loading

### WebSocket Client Implementation

* [ ] Add dependencies to `Cargo.toml`
	* [ ] Add `tokio = { version = "1", features = ["full"] }`
	* [ ] Add `tokio-tungstenite = "0.21"`
	* [ ] Add `serde = { version = "1", features = ["derive"] }`
	* [ ] Add `serde_json = "1"`
	* [ ] Add `tracing = "0.1"`
	* [ ] Add `tracing-subscriber = "0.3"`
* [ ] Create `src/council_client.rs`
	* [ ] Define `CouncilMsg` enum (matching AURA-1)
	* [ ] Define `CouncilEnvelope` struct (matching AURA-1)
	* [ ] Implement `connect_to_council()` function
	* [ ] Return WebSocket stream and sink
* [ ] Implement Hello/Ack handshake in `src/handshake.rs`
	* [ ] Define `Hello` message with `last_ack` field
	* [ ] Send Hello on connection
	* [ ] Wait for server acknowledgment
	* [ ] Store session ID from server
* [ ] Implement replay handling in `src/replay.rs`
	* [ ] Define `ReplayHandler` struct
	* [ ] Collect envelopes until `replay_done` marker
	* [ ] Store replayed envelopes in memory
	* [ ] Log each replayed envelope (no processing yet)
	* [ ] Verify sequence numbers are monotonic
* [ ] Implement live subscription in `src/live.rs`
	* [ ] Define `LiveHandler` struct
	* [ ] Process envelopes after `replay_done`
	* [ ] Log each live envelope (no processing yet)
	* [ ] Update `last_ack` after each envelope
* [ ] Implement reconnection logic in `src/reconnect.rs`
	* [ ] Detect connection loss
	* [ ] Exponential backoff: 1s, 2s, 4s, 8s, max 30s
	* [ ] Send Hello with last known `last_ack`
	* [ ] Resume from last acknowledged sequence

### Main Runtime Loop

* [ ] Create `src/main.rs`
	* [ ] Initialize tracing subscriber
	* [ ] Load `sentinel.json` config
	* [ ] Connect to AURA-1 `/ws/council` endpoint
	* [ ] Perform handshake
	* [ ] Handle replay phase
	* [ ] Subscribe to live envelopes
	* [ ] Log all received messages
	* [ ] Handle reconnects on error
	* [ ] Graceful shutdown on SIGINT/SIGTERM

### Testing

* [ ] Create `tests/sentinel_connection.rs`
	* [ ] Start AURA-1 test server
	* [ ] Start Sentinel runtime
	* [ ] Send test envelope from AURA-1
	* [ ] Verify Sentinel receives and logs it
	* [ ] Verify no authority messages sent by Sentinel
* [ ] Create `tests/sentinel_replay.rs`
	* [ ] Persist 10 envelopes in AURA-1
	* [ ] Start Sentinel with `last_ack: 0`
	* [ ] Verify Sentinel replays all 10 in order
	* [ ] Verify `replay_done` marker received
	* [ ] Send live envelope, verify received after replay
* [ ] Create `tests/sentinel_reconnect.rs`
	* [ ] Start Sentinel, receive 5 envelopes
	* [ ] Kill WebSocket connection
	* [ ] Verify Sentinel reconnects
	* [ ] Verify replay from last ack
	* [ ] Verify no duplicate processing

### Documentation

* [ ] Create `docs/architecture.md`
	* [ ] Explain Sentinel role in Council
	* [ ] Document connection lifecycle
	* [ ] Document replay guarantees
	* [ ] Document reconnection strategy
* [ ] Update README with usage instructions
	* [ ] How to build: `cargo build --release`
	* [ ] How to run: `./target/release/aura-sentinel`
	* [ ] Environment variables: `AURA_COUNCIL_URL`
	* [ ] Expected log output examples

**Stop here. Do not add LLM yet.**

**Invariant:** Archetypes are clients of authority, never sources of it.

---

## PHASE 4 — LLM INFERENCE INTEGRATION (SENTINEL)

> Goal: Add reasoning without compromising determinism.

### Ollama Integration

* [ ] Add dependencies to `Cargo.toml`
	* [ ] Add `reqwest = { version = "0.11", features = ["json"] }`
	* [ ] Add `async-trait = "0.1"`
* [ ] Create `src/llm/mod.rs`
	* [ ] Define `LLMClient` trait
	* [ ] Define `generate()` method signature
	* [ ] Define `LLMRequest` and `LLMResponse` structs
* [ ] Create `src/llm/ollama.rs`
	* [ ] Implement `OllamaClient` struct
	* [ ] Implement `LLMClient` trait for `OllamaClient`
	* [ ] Configure endpoint: `http://localhost:11434/api/generate`
	* [ ] Set model from config (e.g., "llama3:8b")
	* [ ] Set `temperature: 0.0` from config
	* [ ] Set `top_p: 0.1` from config
	* [ ] Implement full response collection (no streaming)
	* [ ] Add timeout: 60 seconds
	* [ ] Add retry logic: 3 attempts with backoff
* [ ] Add Ollama config to `sentinel.json`
	* [ ] Add `llm.provider: "ollama"`
	* [ ] Add `llm.model: "llama3:8b"`
	* [ ] Add `llm.endpoint: "http://localhost:11434"`
	* [ ] Add `llm.temperature: 0.0`
	* [ ] Add `llm.top_p: 0.1`

### Prompt Engineering

* [ ] Create `src/prompts/mod.rs`
	* [ ] Define `build_sentinel_prompt()` function
	* [ ] Accept `CouncilMsg` as input
	* [ ] Return formatted prompt string
* [ ] Create `src/prompts/sentinel_system.txt`
	* [ ] Define Sentinel role: "You are the Sentinel archetype..."
	* [ ] Define constraints: "You MUST NOT override user decisions..."
	* [ ] Define output format: "Respond with analysis in JSON format..."
	* [ ] Define forbidden actions: "You MUST NOT generate authority messages..."
* [ ] Implement prompt builder
	* [ ] Load system prompt from file
	* [ ] Convert `CouncilMsg` to context string
	* [ ] Combine system + context into final prompt
	* [ ] Add test: verify prompt structure

### Response Processing

* [ ] Create `src/response_handler.rs`
	* [ ] Define `process_llm_response()` function
	* [ ] Parse LLM output as JSON
	* [ ] Validate response structure
	* [ ] Extract analysis text
	* [ ] Wrap as `CouncilMsg::Response` variant
	* [ ] Add metadata: archetype name, timestamp
* [ ] Implement response validation
	* [ ] Verify no authority claims in response
	* [ ] Verify no user override attempts
	* [ ] Verify output is analysis, not command
	* [ ] Reject invalid responses (log error, do not broadcast)

### Authority Broadcast Integration

* [ ] Create `src/authority_client.rs`
	* [ ] Implement `broadcast_response()` function
	* [ ] Send `CouncilMsg::Response` to AURA-1
	* [ ] Use `/api/council/broadcast` HTTP endpoint
	* [ ] AURA-1 persists before broadcasting
	* [ ] Wait for acknowledgment from AURA-1
	* [ ] Handle broadcast failures (retry with backoff)
* [ ] Update `src/live.rs` to process and respond
	* [ ] On receiving `CouncilMsg::Query`, extract content
	* [ ] Build prompt from query
	* [ ] Call LLM client
	* [ ] Process response
	* [ ] Broadcast response via authority client
	* [ ] Log full pipeline: query → LLM → response → broadcast

### Testing

* [ ] Create `tests/sentinel_llm_integration.rs`
	* [ ] Start AURA-1 test server
	* [ ] Start Ollama test server (mock)
	* [ ] Start Sentinel runtime
	* [ ] Send `CouncilMsg::Query` from AURA-1
	* [ ] Verify Sentinel calls LLM
	* [ ] Verify Sentinel broadcasts response
	* [ ] Verify response persisted in AURA-1
	* [ ] Verify response replayed on reconnect
* [ ] Create `tests/sentinel_determinism.rs`
	* [ ] Send same query 10 times
	* [ ] Verify responses are identical (temperature=0.0)
	* [ ] Verify sequence numbers are monotonic
* [ ] Create `tests/sentinel_forbidden_domain.rs`
	* [ ] Mock LLM to return authority-claiming response
	* [ ] Verify Sentinel rejects response
	* [ ] Verify no broadcast occurs
	* [ ] Verify error logged

### Documentation

* [ ] Update `docs/architecture.md`
	* [ ] Add LLM integration section
	* [ ] Document prompt structure
	* [ ] Document response validation
	* [ ] Document broadcast flow
* [ ] Create `docs/llm_configuration.md`
	* [ ] Explain Ollama setup
	* [ ] List supported models
	* [ ] Explain temperature and top_p settings
	* [ ] Provide troubleshooting guide

**Invariant:** LLM output explains decisions; it does not make them.

---

## PHASE 5 — DETERMINISTIC VERIFICATION (QSIC PATH)

> Goal: Separate explanation from truth.

### QSIC Implementation

* [ ] Create `src/qsic/mod.rs`
	* [ ] Define `QSICVerifier` struct
	* [ ] Define `verify()` method signature
	* [ ] Accept data bytes and expected hash
	* [ ] Return `VerificationResult` enum
* [ ] Add dependencies to `Cargo.toml`
	* [ ] Add `num-bigint = "0.4"` (arbitrary-precision integers)
	* [ ] Add `sha2 = "0.10"` (SHA-256 hashing)
	* [ ] Add `hex = "0.4"` (hex encoding/decoding)
* [ ] Create `src/qsic/quantum_integer.rs`
	* [ ] Define 21-digit quantum integer constant
	* [ ] Load from config or hardcode: `123456789012345678901`
	* [ ] Document origin: "Derived from Seed of Life geometry..."
	* [ ] Implement `get_quantum_integer()` function
* [ ] Create `src/qsic/hash.rs`
	* [ ] Implement `qsic_hash()` function
	* [ ] Accept: salt (BigInt), data (bytes), quantum_integer (BigInt)
	* [ ] Compute: SHA-256(salt || quantum_integer || data)
	* [ ] Return: hex-encoded hash string
	* [ ] Add test: verify deterministic output
* [ ] Create `src/qsic/verifier.rs`
	* [ ] Implement `QSICVerifier::new()`
	* [ ] Load quantum integer from config
	* [ ] Implement `verify(data, expected_hash, salt)`
	* [ ] Compute QSIC hash of data
	* [ ] Compare with expected hash
	* [ ] Return `VerificationResult::Valid` or `Invalid`
	* [ ] Add test: verify correct data passes
	* [ ] Add test: verify tampered data fails

### Integration with Sentinel

* [ ] Update `src/response_handler.rs`
	* [ ] After receiving LLM response, compute QSIC hash
	* [ ] Attach hash to `CouncilMsg::Response` metadata
	* [ ] Broadcast response with hash
* [ ] Create `src/verdict_generator.rs`
	* [ ] Define `generate_verdict()` function
	* [ ] Accept `CouncilMsg::Response` and original query
	* [ ] Verify QSIC hash of response
	* [ ] If valid, wrap as `CouncilMsg::Verdict`
	* [ ] If invalid, wrap as `CouncilMsg::Refusal` with reason
	* [ ] Broadcast verdict via authority client
* [ ] Update `src/live.rs` to generate verdicts
	* [ ] After broadcasting response, generate verdict
	* [ ] Broadcast verdict as separate envelope
	* [ ] Log verdict generation

### Testing

* [ ] Create `tests/qsic_verification.rs`
	* [ ] Test QSIC hash computation
	* [ ] Test verification with correct hash
	* [ ] Test verification with incorrect hash
	* [ ] Test verification with tampered data
* [ ] Create `tests/sentinel_verdict_flow.rs`
	* [ ] Send query to Sentinel
	* [ ] Verify Sentinel broadcasts response with QSIC hash
	* [ ] Verify Sentinel broadcasts verdict after verification
	* [ ] Verify verdict references response sequence number
	* [ ] Verify verdict persisted in AURA-1
* [ ] Create `tests/sentinel_verdict_ordering.rs`
	* [ ] Send query to Sentinel
	* [ ] Verify response persisted before verdict
	* [ ] Verify verdict references response
	* [ ] Verify ordering is inviolable on replay

### Documentation

* [ ] Create `docs/qsic.md`
	* [ ] Explain QSIC concept
	* [ ] Document quantum integer derivation
	* [ ] Document hash computation algorithm
	* [ ] Provide examples
* [ ] Update `docs/architecture.md`
	* [ ] Add verdict generation section
	* [ ] Document response → verdict flow
	* [ ] Document ordering guarantees

**Invariant:** Truth is computed, not predicted.

---

## PHASE 6 — ARCHITECT COORDINATION

> Goal: Controlled multi-archetype activation.

### Architect Archetype Setup

* [ ] Create new repository: `NeuroCognica/aura-architect`
	* [ ] Initialize Rust project
	* [ ] Copy WebSocket client code from Sentinel
	* [ ] Create `config/architect.json`
* [ ] Define Architect config
	* [ ] Set `temperature: 0.3` (moderate creativity)
	* [ ] Set `top_p: 0.5`
	* [ ] Define role: "Coordination and task decomposition"
	* [ ] Define forbidden domains: ["user_override", "sentinel_override"]
	* [ ] Define allowed message types: ["activation", "deliberation"]

### Activation Logic

* [ ] Create `src/activation.rs`
	* [ ] Define `ActivationDecision` struct
	* [ ] Fields: target_archetype, reason, priority
	* [ ] Implement `decide_activation()` function
	* [ ] Accept user message as input
	* [ ] Call LLM to analyze message
	* [ ] Parse LLM output to extract activation decision
	* [ ] Return list of archetypes to activate
* [ ] Create `src/archetype_registry.rs`
	* [ ] Define `ArchetypeRegistry` struct
	* [ ] List all available archetypes: Sentinel, Explorer, Jester, etc.
	* [ ] Define archetype capabilities and domains
	* [ ] Implement `get_archetype_for_task()` function
* [ ] Implement activation broadcasting
	* [ ] Wrap activation decision as `CouncilMsg::Activation`
	* [ ] Broadcast via AURA-1 authority spine
	* [ ] Target archetypes listen for their activation
	* [ ] Log activation decision

### Deliberation Protocol

* [ ] Create `src/deliberation.rs`
	* [ ] Define `Deliberation` message type
	* [ ] Fields: initiator, target, disagreement_reason
	* [ ] Implement `initiate_deliberation()` function
	* [ ] Architect calls this when archetypes disagree
	* [ ] Broadcast deliberation message
	* [ ] Wait for archetype responses
	* [ ] Summarize deliberation for user
	* [ ] User makes final decision
* [ ] Update `src/live.rs` to handle deliberations
	* [ ] Listen for `CouncilMsg::Deliberation`
	* [ ] If Architect is target, respond with reasoning
	* [ ] Never override Sentinel verdicts
	* [ ] Log all deliberations

### Testing

* [ ] Create `tests/architect_activation.rs`
	* [ ] Start AURA-1, Sentinel, and Architect
	* [ ] Send user message to Architect
	* [ ] Verify Architect broadcasts activation decision
	* [ ] Verify Sentinel receives activation
	* [ ] Verify Sentinel processes message
* [ ] Create `tests/architect_deliberation.rs`
	* [ ] Simulate disagreement between Architect and Sentinel
	* [ ] Verify Architect initiates deliberation
	* [ ] Verify Sentinel responds
	* [ ] Verify deliberation logged
	* [ ] Verify user presented with summary
* [ ] Create `tests/architect_no_override.rs`
	* [ ] Sentinel issues verdict
	* [ ] Architect disagrees
	* [ ] Verify Architect cannot override verdict
	* [ ] Verify deliberation initiated instead

### Documentation

* [ ] Create `docs/architect.md`
	* [ ] Explain Architect role
	* [ ] Document activation protocol
	* [ ] Document deliberation protocol
	* [ ] Provide examples
* [ ] Update `docs/architecture.md`
	* [ ] Add multi-archetype coordination section
	* [ ] Document activation flow
	* [ ] Document deliberation flow

**Invariant:** The system deliberates; the human decides.

---

## PHASE 7 — FULL COUNCIL (OPTIONAL, SEQUENTIAL)

> Goal: Expand cognition without breaking law.

### For Each Archetype: Explorer, Jester, Mentor, Empath, Oracle

#### Archetype Setup

* [ ] Create repository: `NeuroCognica/aura-{archetype}`
* [ ] Initialize Rust project
* [ ] Copy WebSocket client code
* [ ] Create `config/{archetype}.json`
	* [ ] Define temperature (varies by archetype)
	* [ ] Define top_p (varies by archetype)
	* [ ] Define role description
	* [ ] Define forbidden domains
	* [ ] Define allowed message types

#### Archetype-Specific Implementation

* [ ] **Explorer**
	* [ ] Temperature: 0.7 (high creativity)
	* [ ] Role: "Explore novel solutions and unconventional approaches"
	* [ ] Forbidden: ["user_override", "sentinel_override", "reckless_action"]
	* [ ] Implement exploration prompt templates
	* [ ] Implement novelty scoring
	* [ ] Test: Verify Explorer suggests creative solutions
	* [ ] Test: Verify Explorer respects safety boundaries
* [ ] **Jester**
	* [ ] Temperature: 0.9 (maximum creativity)
	* [ ] Role: "Challenge assumptions and provide contrarian perspectives"
	* [ ] Forbidden: ["user_override", "sentinel_override", "mockery"]
	* [ ] Implement contrarian prompt templates
	* [ ] Implement assumption identification
	* [ ] Test: Verify Jester challenges group consensus
	* [ ] Test: Verify Jester remains respectful
* [ ] **Mentor**
	* [ ] Temperature: 0.2 (low, pedagogical)
	* [ ] Role: "Provide guidance and educational context"
	* [ ] Forbidden: ["user_override", "sentinel_override", "coercion"]
	* [ ] Implement teaching prompt templates
	* [ ] Implement knowledge assessment
	* [ ] Test: Verify Mentor provides explanations
	* [ ] Test: Verify Mentor does not coerce decisions
* [ ] **Empath**
	* [ ] Temperature: 0.4 (moderate, empathetic)
	* [ ] Role: "Assess emotional and social implications"
	* [ ] Forbidden: ["user_override", "sentinel_override", "manipulation"]
	* [ ] Implement empathy prompt templates
	* [ ] Implement sentiment analysis
	* [ ] Test: Verify Empath assesses emotional impact
	* [ ] Test: Verify Empath does not manipulate
* [ ] **Oracle**
	* [ ] Temperature: 0.1 (very low, predictive)
	* [ ] Role: "Provide long-term forecasting and strategic analysis"
	* [ ] Forbidden: ["user_override", "sentinel_override", "certainty_claims"]
	* [ ] Implement forecasting prompt templates
	* [ ] Implement uncertainty quantification
	* [ ] Test: Verify Oracle provides forecasts
	* [ ] Test: Verify Oracle defers to Witness for decisions

#### Integration Testing

* [ ] Create `tests/full_council_integration.rs`
	* [ ] Start AURA-1 and all seven archetypes
	* [ ] Send complex user query
	* [ ] Verify Architect activates appropriate archetypes
	* [ ] Verify each archetype responds in character
	* [ ] Verify Sentinel issues final verdict
	* [ ] Verify all messages persisted in order
	* [ ] Verify replay works for all archetypes

#### Documentation

* [ ] Create `docs/{archetype}.md` for each archetype
* [ ] Update `docs/architecture.md` with full Council description
* [ ] Create `docs/council_dynamics.md`
	* [ ] Explain how archetypes interact
	* [ ] Document thermal stratification
	* [ ] Document deliberation patterns
	* [ ] Provide example conversations

**Invariant:** New minds must not weaken the constitution.

---

## PHASE 8 — CRYPTOGRAPHIC ATTESTATION (FUTURE)

> Goal: Prevent forgery, not just disorder.

### Cryptographic Infrastructure

* [ ] Add dependencies to AURA-1 `Cargo.toml`
	* [ ] Add `ed25519-dalek = "2.0"`
	* [ ] Add `rand = "0.8"`
* [ ] Create `backend/src/crypto/mod.rs`
	* [ ] Define `Keypair` struct
	* [ ] Define `Signature` struct
	* [ ] Implement key generation
	* [ ] Implement signing
	* [ ] Implement verification
* [ ] Create `backend/src/crypto/keystore.rs`
	* [ ] Implement secure key storage
	* [ ] Store keys in `data/keys/{archetype}.key`
	* [ ] Encrypt keys at rest (use `age` or similar)
	* [ ] Implement key loading on startup
	* [ ] Implement key rotation protocol

### Signature Integration

* [ ] Update `CouncilEnvelope` struct
	* [ ] Add `signature: Option<Vec<u8>>` field
	* [ ] Add `signer_pubkey: Option<Vec<u8>>` field
* [ ] Update `make_council_envelope()` function
	* [ ] Accept keypair as parameter
	* [ ] Sign envelope content
	* [ ] Attach signature and public key
* [ ] Update `append_council_envelope_with()` function
	* [ ] Verify signature before persisting
	* [ ] Reject unsigned envelopes (after migration period)
	* [ ] Reject invalid signatures
	* [ ] Log signature verification failures
* [ ] Update `load_council_range()` function
	* [ ] Verify signatures on load
	* [ ] Reject tampered envelopes
	* [ ] Log integrity violations

### Archetype Key Management

* [ ] Generate keypair for each archetype
	* [ ] Sentinel keypair
	* [ ] Architect keypair
	* [ ] Explorer keypair
	* [ ] Jester keypair
	* [ ] Mentor keypair
	* [ ] Empath keypair
	* [ ] Oracle keypair
* [ ] Distribute public keys to AURA-1
	* [ ] Store in `data/pubkeys/{archetype}.pub`
	* [ ] Load on AURA-1 startup
	* [ ] Verify all messages against known public keys
* [ ] Update archetypes to sign messages
	* [ ] Load private key on startup
	* [ ] Sign all outgoing `CouncilMsg` envelopes
	* [ ] Attach signature before broadcasting

### Testing

* [ ] Create `backend/tests/signature_verification.rs`
	* [ ] Generate test keypair
	* [ ] Sign test envelope
	* [ ] Verify signature
	* [ ] Tamper with envelope
	* [ ] Verify signature fails
* [ ] Create `backend/tests/unsigned_rejection.rs`
	* [ ] Attempt to persist unsigned envelope
	* [ ] Verify rejection
	* [ ] Verify error logged
* [ ] Create `tests/archetype_signature_integration.rs`
	* [ ] Start AURA-1 and Sentinel
	* [ ] Sentinel sends signed message
	* [ ] Verify AURA-1 accepts and persists
	* [ ] Tamper with message in transit (mock)
	* [ ] Verify AURA-1 rejects tampered message

### Documentation

* [ ] Create `docs/cryptography.md`
	* [ ] Explain signature scheme
	* [ ] Document key generation
	* [ ] Document key storage
	* [ ] Document verification process
* [ ] Create `docs/security_model.md`
	* [ ] Document threat model
	* [ ] Document attack vectors
	* [ ] Document mitigations
	* [ ] Document incident response

**Invariant:** Authority must be provable under adversarial conditions.

---

## PHASE 9 — OPTIONAL DISTRIBUTION / REPLICATION

> Goal: Survive process failure.

### Replication Strategy

* [ ] Evaluate replication options
	* [ ] Raft consensus (via `tikv/raft-rs`)
	* [ ] Multi-master replication
	* [ ] Leader-follower replication
	* [ ] Decide on strategy based on requirements
* [ ] Add dependencies to AURA-1 `Cargo.toml`
	* [ ] Add `raft = "0.7"` (if using Raft)
	* [ ] Add `tokio-raft` or equivalent
* [ ] Create `backend/src/replication/mod.rs`
	* [ ] Define `ReplicationManager` struct
	* [ ] Implement leader election
	* [ ] Implement log replication
	* [ ] Implement follower synchronization

### Snapshot and Restore

* [ ] Create `backend/src/snapshot.rs`
	* [ ] Implement RocksDB snapshot creation
	* [ ] Store snapshots in `data/snapshots/{timestamp}.snap`
	* [ ] Implement snapshot compression
	* [ ] Implement snapshot cleanup (keep last N)
* [ ] Create `backend/src/restore.rs`
	* [ ] Implement snapshot restoration
	* [ ] Verify snapshot integrity before restore
	* [ ] Replay envelopes after snapshot point
	* [ ] Verify final state matches expected
* [ ] Add snapshot scheduling
	* [ ] Snapshot every 1 hour (configurable)
	* [ ] Snapshot on graceful shutdown
	* [ ] Snapshot before major operations

### Failover Testing

* [ ] Create `backend/tests/failover_integration.rs`
	* [ ] Start AURA-1 leader
	* [ ] Persist 100 envelopes
	* [ ] Kill leader process
	* [ ] Start follower, promote to leader
	* [ ] Verify follower has all 100 envelopes
	* [ ] Verify sequence numbers unchanged
	* [ ] Send new envelope to new leader
	* [ ] Verify persistence and broadcast
* [ ] Create `backend/tests/snapshot_restore.rs`
	* [ ] Persist 100 envelopes
	* [ ] Create snapshot
	* [ ] Persist 50 more envelopes
	* [ ] Restore from snapshot
	* [ ] Replay 50 envelopes
	* [ ] Verify final state correct

### Documentation

* [ ] Create `docs/replication.md`
	* [ ] Explain replication strategy
	* [ ] Document leader election
	* [ ] Document failover procedure
	* [ ] Provide operational runbook
* [ ] Create `docs/disaster_recovery.md`
	* [ ] Document backup procedures
	* [ ] Document restore procedures
	* [ ] Document data loss scenarios
	* [ ] Provide recovery checklists

---

## PHASE 10 — SSSD INTEGRATION (FUTURE)

> Goal: Govern the Solid-State Spacetime Drive.

### SSSD Simulator

* [ ] Create `sssd-simulator` crate
	* [ ] Implement Diamond Tetrahedron resonance model
	* [ ] Implement QSIC verification in drive control
	* [ ] Implement simulated thrust output
	* [ ] Implement safety interlocks
	* [ ] Expose control API

### AURA-SSSD Bridge

* [ ] Create `aura-sssd-bridge` crate
	* [ ] Connect to AURA-1 `/ws/council`
	* [ ] Connect to SSSD simulator API
	* [ ] Translate `CouncilMsg::Command` to SSSD control
	* [ ] Require Sentinel verdict before execution
	* [ ] Implement emergency shutdown on refusal
	* [ ] Log all control actions

### Safety Verification

* [ ] Implement pre-flight checks
	* [ ] Verify QSIC integrity of drive firmware
	* [ ] Verify all archetypes online
	* [ ] Verify Sentinel verdict for activation
	* [ ] Verify no deliberations pending
* [ ] Implement runtime monitoring
	* [ ] Monitor drive telemetry
	* [ ] Detect anomalies
	* [ ] Broadcast anomalies to Council
	* [ ] Sentinel issues verdict on anomaly
	* [ ] Shutdown if verdict is refusal

### Testing

* [ ] Create `tests/sssd_integration.rs`
	* [ ] Start AURA-1, Sentinel, and SSSD simulator
	* [ ] Send activation command
	* [ ] Verify Sentinel issues verdict
	* [ ] Verify SSSD activates only after verdict
	* [ ] Simulate anomaly
	* [ ] Verify Sentinel detects and refuses
	* [ ] Verify SSSD shuts down

### Documentation

* [ ] Create `docs/sssd_integration.md`
	* [ ] Explain SSSD control architecture
	* [ ] Document safety protocols
	* [ ] Document emergency procedures
	* [ ] Provide operational checklists

---

## FINAL RULES (DO NOT DELETE)

* Authority > Cognition > UX
* Persist before broadcast
* Replay before live
* Refusal is success
* Determinism beats persuasion
* Law does not optimize for comfort

---

## CODING AGENT INSTRUCTIONS

### General Principles

1. **Never skip phases.** Each phase builds on the previous. Complete all tasks in a phase before advancing.
2. **Test everything.** Every feature must have at least one integration test.
3. **Document as you go.** Update docs immediately after implementing features.
4. **Preserve invariants.** Every phase has an invariant. If your code violates it, revert and redesign.
5. **Fail fast.** If a test fails, stop and fix it before proceeding.
6. **No silent failures.** All errors must be logged with context.
7. **Determinism first.** If a feature introduces non-determinism, it is wrong.
8. **Authority is sacred.** Never bypass the authority spine. Never cache authority messages. Never reorder them.

### Code Style

* Use `rustfmt` and `clippy` on all code.
* Write descriptive commit messages: "Add QSIC verification to Sentinel" not "fix bug".
* Use `tracing` for all logging, not `println!`.
* Use `anyhow` for error handling in applications, `thiserror` for libraries.
* Prefer explicit types over `impl Trait` in public APIs.
* Document all public functions with `///` doc comments.

### Testing Strategy

* Unit tests in `src/` files using `#[cfg(test)]` modules.
* Integration tests in `tests/` directory.
* Use `tokio::test` for async tests.
* Use temporary directories for test databases: `tempfile::tempdir()`.
* Clean up resources in tests: use `Drop` or explicit cleanup.
* Run tests with `cargo test --all-features` before committing.

### Git Workflow

* Work on feature branches: `feature/phase-3-sentinel-runtime`.
* Commit frequently with descriptive messages.
* Push to GitHub after each completed task.
* Create PR when phase is complete.
* Require CI to pass before merging.
* Tag releases: `aura-sentinel-v0.1.0`.

### Deployment

* Build release binaries: `cargo build --release`.
* Store binaries in `bin/` directory.
* Use systemd for process management in production.
* Store data in `/var/lib/aura/` in production.
* Store logs in `/var/log/aura/` in production.
* Rotate logs daily, keep 30 days.

---

This checklist is machine-readable and actionable. Use it as the single source of truth for implementation phases and completion criteria.
