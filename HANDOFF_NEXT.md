Handoff report — next chat instance
Date: 2025-12-31

Purpose
-------
This document is a single-file handoff for the next developer (or next chat instance). It describes what this project is, what we have built (including the recent "authority spine" work), the guarantees now enforced by code and tests, how to build/run/verify locally, and a prioritized list of remaining work and suggested next actions.

Project summary
---------------
- Repository: AURA-1 (Rust backend + stubbed frontend)
- Primary runtime: Rust async binary (Axum + Tokio). Backend is authoritative for state and persistence.
- Persistence: RocksDB (persistence feature). Tantivy used for search (optional feature).
- Purpose: Provide a governed intelligence substrate with temporal authority for council messages, replayable event streams, and robust persistence and enforcement hooks.

What we built (summary)
-----------------------
- Typed council contract: `CouncilMsg` enum + `CouncilEnvelope` struct and helper `make_council_envelope`.
- Storage helpers: `append_council_envelope_with`, `get_council_envelope`, `load_council_range` in `backend/src/storage.rs` to atomically persist typed envelopes and allocate per-session sequence numbers.
- Persist-first broadcast pipeline: `broadcast_council` (in `backend/src/broadcast.rs`) now persists envelopes first, emits the legacy JSON string for backwards compatibility, and also publishes a typed `CouncilEnvelope` on a typed broadcast channel.
- Typed WS handler: `/ws/council` now supports a Hello/Ack handshake, replay of persisted envelopes (bounded), an explicit `replay_done` marker, and then deterministic live forwarding of typed envelopes. The handler preserves a legacy string lane as fallback.
- Tests: Added unit serialization test and a full end-to-end integration test `backend/tests/ws_replay_integration.rs` that proves replay order, `replay_done` boundary, and live delivery semantics.
- Releases/branching: Committed, pushed, and tagged the milestone as `aura-1-authority-spine-v1` (tag exists on remote). A PR branch `authority-spine/v1` was created and then fast-forward merged into `main`.

Design guarantees (now enforced)
-------------------------------
- Replay order: persisted sequence is authoritative and deterministic.
- Replay boundary: explicit `replay_done` message is emitted; live deliveries occur only after replay finishes.
- No race between replay and live: storage-owned sequencing and typed channel ordering prevent race conditions.
- No duplicate/phantom envelopes: per-session seq allocated in storage and stored alongside envelope bytes.
- Auditability: persisted envelopes, `sess_council_last:{sid}`, and range reads make MMR/integrity reviews straightforward.

Key files & symbols
-------------------
- `backend/src/council_verdict.rs` — typed council message definitions, `CouncilMsg`, `CouncilEnvelope`, `make_council_envelope`.
- `backend/src/storage.rs` — RocksDB helper methods: `append_council_envelope_with`, `get_council_envelope`, `load_council_range`.
- `backend/src/broadcast.rs` — `broadcast_council`, legacy string broadcast preservation, typed broadcast channel, `ws_council_handler`, `handle_council_socket_typed` with Hello/Ack/replay/replay_done.
- `backend/src/main.rs` — DI wiring: typed broadcast channel exposed as `Extension` for handlers.
- `backend/tests/ws_replay_integration.rs` — end-to-end test proving replay semantics.
- Release notes: `RELEASE_NOTES/aura-1-authority-spine-v1.md` and PR files `PR_AUTHORITY_SPINE_V1.md`, `PR_BODY_AURA_AUTHORITY_SPINE_V1.md` exist in repo root.

How to build and run locally
----------------------------
Prerequisites (dev machine Windows example):
- Rust toolchain (stable) installed via rustup
- `cmake` and `nasm` in PATH (native build deps for some crates)
- `cargo` and `git` available

Build (release):
```powershell
cd C:\AURA-1
cargo build --release
```

Run dev binary (features):
```powershell
cd C:\AURA-1
cargo run --features "persistence search tls"
```

Run tests (complete backend suite including integration):
```powershell
cd C:\AURA-1\backend
cargo test --features "persistence search tls" -- --nocapture
```

Run only the replay integration test:
```powershell
cd C:\AURA-1\backend
cargo test --test ws_replay_integration --features "persistence search tls" -- --nocapture
```

How the WS replay test works (quick):
- Creates a temporary RocksDB store and persists N typed envelopes via `append_council_envelope_with`.
- Starts a local Axum server exposing `/ws/council` handler.
- Connects a WebSocket client, sends a Hello with `last_ack` set to a lower seq, expects persisted envelopes replayed in order, expects `replay_done`, then verifies live envelope arrives.

Operational notes / gotchas
--------------------------
- Native build tools: missing `cmake` or `nasm` will cause build failures; ensure they are in PATH on Windows (or install via package manager).
- Warnings: Some compiler warnings (unused vars, deprecated base64 usage) are present and not fatal; they should be reviewed but do not block the milestone.
- Backward compatibility: legacy JSON string broadcast is intentionally preserved. Plan a coordinated client migration before removal.

Commits & tags (milestone)
---------------------------
- Milestone tag: `aura-1-authority-spine-v1` (pushed to remote).
- Recent merges: `authority-spine/v1` branch pushed and fast-forward merged into `main` (commit fb60c6d pushed).
- PR branch: `authority-spine/v1` exists (was used for review docs); PR files are present in repo root.

Remaining work (recommended order)
---------------------------------
Short-term (small, safe):
- Add metrics for replay rate, lag, and last persisted seq per session (prometheus endpoints). (effort: small)
- Add integration to CI to run `ws_replay_integration` on feature-enabled runs and fail on regressions. (effort: small)
- Replace deprecated `base64::encode` usage with modern Engine API. (effort: tiny)

Mid-term (medium effort):
- Deprecate and remove legacy JSON string broadcast after client migration plan is in place. (effort: medium — coordination required)
- Add MMR proofs for persisted envelopes if cryptographic auditability is required (the repo already uses RocksDB; build MMR commits). (effort: medium)
- Add end-to-end scenario tests for client reconnect and large replay windows. (effort: medium)

Long-term (large):
- Expose operational dashboards and alerts for replay failures, database errors, and lagging subscribers. (effort: large)
- Harden persistence migrations (if evolving storage keys or formats, add migration tooling). (effort: large)

Suggested next steps for the next chat instance
----------------------------------------------
1. Decide branch housekeeping: delete `authority-spine/v1` remote branch if no longer needed.
2. Add CI job to run the replay integration test on each push to `main` (feature-enabled matrix). I can draft the workflow if desired.
3. Start a short follow-up to collect metrics and add a Prometheus scrape endpoint around `backend/src/broadcast.rs` and `storage.rs` (expose `sess_council_last:{sid}` as metric). I can add scaffolding.
4. If you want to remove legacy string broadcasts, plan a migration: make a note in `PR_BODY_AURA_AUTHORITY_SPINE_V1.md`, and coordinate client updates (you’re solo, so you can do this directly).

Troubleshooting checklist
-------------------------
- Build fails with missing `cmake`/`nasm`: install them and re-run `cargo test`.
- WebSocket test fails with timeouts: increase timeout durations in `backend/tests/ws_replay_integration.rs` and ensure no port collision.
- RocksDB errors: ensure tests use ephemeral temp dirs (they do); check file locks on Windows.

Contacts & ownership
--------------------
- This workspace is single-owner (you). All decisions, merges, and removals are under your control. For auditing or external reviewers, consult the release notes file `RELEASE_NOTES/aura-1-authority-spine-v1.md` and PR body files in the repo root.

Files added for this milestone
-----------------------------
- `backend/tests/ws_replay_integration.rs` — E2E replay test
- `backend/tests/council_envelope_serde.rs` — serialization test
- `PR_AUTHORITY_SPINE_V1.md` — PR summary
- `PR_BODY_AURA_AUTHORITY_SPINE_V1.md` — PR body (ready to paste)
- `RELEASE_NOTES/aura-1-authority-spine-v1.md`

Final note
----------
This milestone (authority spine) converts convention to compiler-and-test-enforced guarantees. It is deliberately the last foundational change before layering additional features. Treat the `aura-1-authority-spine-v1` tag as the canonical constitutional baseline; subsequent work should be additive and accompanied by tests that preserve the temporal guarantees.

Next action I can take now
-------------------------
- (A) Delete the remote branch `authority-spine/v1`
- (B) Draft CI workflow that runs the replay integration test for pushes to `main`
- (C) Add Prometheus metrics scaffolding for replay/storage metrics

Tell me which one you want me to do and I will proceed.
