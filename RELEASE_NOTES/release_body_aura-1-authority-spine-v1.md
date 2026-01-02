AURA-1 — Authority Spine v1

This release finalizes the constitutional authority layer of AURA-1.

It marks the point where authority decisions become explicit, persisted, replayable, and enforceable across time, rather than transient runtime events.

What is now true

- Council authority is expressed as typed `CouncilEnvelope` artifacts.

Every authoritative decision:
- is sequence-owned by storage
- is persisted before broadcast
- cannot exist only in memory

Authority survives:
- server restarts
- client reconnects
- WebSocket drops

- `/ws/council` is the sole authoritative channel
- `/ws/ai` is non-authoritative by design

Enforcement guarantees

- Generation is cancelled immediately on deny / interrupt
- Cancelled generations cannot resume
- Client UX cannot bypass authority
- Authority is never inferred from tokens or chat output

Temporal guarantees

- Clients reconnect using a hello / `last_ack` handshake
- Missing authority envelopes are replayed in strict order
- Live authority does not stream until replay completes
- An explicit `replay_done` boundary separates history from live state

Tests

- Deterministic serde tests for authority envelopes
- Persistence and sequencing tests
- End-to-end WebSocket integration test asserting:
  - correct replay order
  - no duplication
  - no live-before-replay behavior

Scope

This release intentionally does not include:
- new archetypes
- UX changes
- performance optimizations
- softening of Sentinel behavior

Those layers come later. This release exists to make authority durable, inspectable, and non-negotiable.

Tag: aura-1-authority-spine-v1
Commit: fb60c6d

What to Do After This (Minimal, Calm)

- Publish the release.
- Delete the branch (clean history) if you no longer need it.
- Do nothing for a moment — let this settle as a fixed point.

From here on out, everything you build stands on bedrock, not wet clay.
