# PR: Typed, persisted, replayable council authority

Summary
-------
This PR stabilizes the authority spine for council messages by introducing:

- Typed council contract: `CouncilMsg` and `CouncilEnvelope` with a helper `make_council_envelope`.
- Persist-first semantics: `append_council_envelope_with` assigns per-session sequence numbers in storage and persists envelopes.
- A persist-first `broadcast_council` that publishes both the legacy JSON string and a typed `CouncilEnvelope` on a typed broadcast channel.
- A typed `/ws/council` WebSocket handler that enforces a Hello/Ack handshake, replays persisted envelopes in strict order (bounded), emits an explicit `replay_done` marker, then switches to live typed forwarding.
- An end-to-end integration test `backend/tests/ws_replay_integration.rs` that proves replay order, `replay_done` semantics, and live delivery post-replay.

Why this change
---------------
This commit converts the council broadcast path from a best-effort, discipline-based model into a provable, test-locked authority. The integration test enforces the temporal guarantees that are critical to downstream auditing and replay semantics.

What to review
-------------
- `backend/src/council_verdict.rs`: typed contract and constructors
- `backend/src/storage.rs`: `append_council_envelope_with`, `get_council_envelope`, `load_council_range`
- `backend/src/broadcast.rs`: persist-first broadcast and typed WS handler
- `backend/tests/ws_replay_integration.rs`: end-to-end test validating replay and live boundaries

Testing
-------
Run the full backend test suite with persistence/search features:

```powershell
cd backend
cargo test --features "persistence search tls" -- --nocapture
```

Merge strategy
--------------
- Fast-forward or merge with a single commit; this change is self-contained and additive.
- After merge: consider opening a follow-up to remove legacy string broadcast once clients have migrated.

Suggested reviewers
-------------------
- @lead-backend (ownership of persistence and sequencing)
- @security (audit for integrity guarantees)
- @frontend-lead (to coordinate client migration plan)

Suggested labels
----------------
- type:feature
- area:persistence
- area:replay
- milestone:authority-spine-v1

