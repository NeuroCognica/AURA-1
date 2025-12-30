# Council JSON Contract & Client Migration Guide

This document records the canonical JSON envelope shape emitted on the `/ws/council` transport and explains client-side expectations for migration to the Authority State Machine (ASM).

## Envelope shape

All messages on `/ws/council` MUST be JSON objects with these top-level fields:

- `seq` (number): monotonically increasing sequence for the session.
- `type` (string): one of `verdict`, `appeal_state`, `interrupt`.
- `sid` (string): session id.
- `payload` (object): type-specific payload.

Example (verdict):

```json
{
  "seq": 42,
  "type": "verdict",
  "sid": "session-abc123",
  "payload": {
    "verdict_id": "v42",
    "decision": "require_consent",
    "severity": "high",
    "constraints": {"foo": "bar"},
    "require_consent": {
      "phrase": "I understand and confirm",
      "scope": {"action": "dangerous_op"}
    }
  }
}
```

Example (interrupt):

```json
{
  "seq": 43,
  "type": "interrupt",
  "sid": "session-abc123",
  "payload": {
    "reason_code": "safety_pause",
    "scope": {"area": "generation"}
  }
}
```

Example (appeal_state):

```json
{
  "seq": 44,
  "type": "appeal_state",
  "sid": "session-abc123",
  "payload": {
    "state": "authorized",
    "override_token": {"token": "tok", "expires_at_ms": 1710000000000}
  }
}
```

## Client migration notes

- The client must adopt the Authority State Machine (ASM). The recommended stack is Electron + React + TypeScript + Redux Toolkit.
- All authority changes MUST be applied only via the `/ws/council` ingestion point (`CouncilClient`), which translates envelopes into typed `applyCouncilEvent` actions.
- `/ws/ai` remains a token stream only and must not mutate authority.
- Implement `authoritySlice` and `authorityGate` as the canonical enforcement surface. Token rendering must call `authorityGate` at render time and drop tokens if denied.
- If `/ws/council` disconnects, clients MUST reset authority state to a safe default (fail-closed).

## Rendering rules for clients

- `verdict` decisions:
  - `allow`: rendering and input allowed.
  - `allow_with_warning`: rendering allowed; UI may show warning.
  - `require_consent`: rendering blocked until consent flow completes; only `submit_consent` action allowed.
  - `deny`: fully blocked.

- `interrupt` is advisory unless a later `verdict` escalates; clients should treat interrupts as transient and not change authority except via `/ws/council` verdicts.

## Backwards compatibility

- Servers may mirror a minimal advisory notice to `/ws/ai` for migration convenience, but this MUST NOT be authoritative. Clients must ignore such notices for authority.

## Next steps for server teams

- Consider migrating `council_bcast` to a typed transport `Sender<CouncilEnvelope>` to avoid JSON drift.
- Optionally add server-side generation enforcement hooks to consult persisted council state before starting generation.
