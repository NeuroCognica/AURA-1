# Release Notes — aura-1-authority-spine-v1

Date: 2025-12-31

Summary
-------
This release introduces the stabilized authority spine for council messaging.

Highlights
----------
- Typed, persisted council envelopes (`CouncilEnvelope`, `CouncilMsg`).
- Persist-first broadcast semantics guaranteeing storage owns sequencing.
- `/ws/council` handshake with Hello/Ack, deterministic replay window, and explicit `replay_done` marker.
- Integration test that verifies replay order and live-delivery guarantees.

Migration notes
---------------
- Legacy string broadcast is still emitted for backwards compatibility. Clients should migrate to typed `CouncilEnvelope` and the `/ws/council` handshake.

How to verify locally
---------------------
Run the backend tests with persistence and search features enabled:

```powershell
cd backend
cargo test --features "persistence search tls" -- --nocapture
```

