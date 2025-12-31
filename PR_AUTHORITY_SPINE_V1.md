**Title**: Typed, persisted, replayable council authority

- **Summary**: This PR introduces the authority spine for council messages: typed `CouncilEnvelope` payloads, persist-first semantics, and a replayable `/ws/council` socket with Hello/Ack and explicit `replay_done` boundary.
- **Key changes**:
  - Add typed council contract (`CouncilMsg`, `CouncilEnvelope`) and `make_council_envelope` constructor.
  - Add storage helper `append_council_envelope_with` to atomically assign per-session sequence numbers and persist envelopes.
  - Update `broadcast_council` to persist-first and publish typed envelopes on a typed broadcast channel while retaining legacy JSON string broadcasts for compatibility.
  - Implement typed `/ws/council` handler that enforces Hello/Ack handshake, replays persisted envelopes in order (bounded), emits a `replay_done` marker, and then switches to live typed forwarding.
  - Add end-to-end integration test `backend/tests/ws_replay_integration.rs` that verifies replay order, `replay_done`, and live delivery.

- **Why this matters**: The integration test locks temporal semantics so replay order, replay boundary, and live-vs-replay are compiler- and test-enforced guarantees.

- **Testing**: `cargo test --features "persistence search tls"` — unit and integration tests pass in CI/dev.

- **Notes**: Legacy string broadcast is preserved for backward compatibility; removal is deliberately deferred.
