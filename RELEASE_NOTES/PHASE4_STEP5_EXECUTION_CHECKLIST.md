# Phase 4 · Step 5 — Execution Enablement: Checklist & Rollback Plan

Status: DRAFT (planning only — NO IMPLEMENTATION)
Date: 2025-12-31
Related tag: v0.4.4-orchestrator-safe-cognition (precondition)

Purpose
-------
Define the exact, auditable conditions under which real execution (side-effecting actions) may be enabled. This document is a gating artifact: it does not implement execution, it defines the approval, safety, observability, and rollback controls required before any code change enabling `ExecutionMode::Live` is allowed.

Core constraints
----------------
- Only one *narrow* action type is permitted for the initial enablement (see Action Spec).
- Sentinel approval is mandatory and must be explicit, auditable, and non-ambiguous.
- Human visibility and operator confirmation are required before, during, and after execution.
- A hard, out-of-band kill-switch must exist that does not depend on any LLM behavior.
- Any enablement must be reversible or have a documented compensating rollback.
- All proof artifacts (logs, MMR roots, checksums, digital signatures) are mandatory and preserved.

1) Action Specification (single executable action)
-------------------------------------------------
- Define exactly one atomic action allowed in Step 5. Example (illustrative only):
  - `WriteArtifactToNotepadVault` — append a text artifact to an *immutable* audit-only vault (no external system effects).
- Action must be:
  - Single-verb, single-domain, and minimal-surface-area (no external network calls, no shell execution, no privileged OS actions).
  - Fully specified with input schema and size limits.
  - Idempotent where possible or carry an explicit idempotency token.

2) Sentinel Approval Semantics
-----------------------------
- Approval must be a clearly typed `CouncilMsg::Verdict` with enumerated state `Approve` and an explicit `verdict_metadata` payload including:
  - `verdict_id`, `proposed_action_id`, `timestamp`, `sentinel_model`, `system_prompt_hash`, `signature`.
- `Approve` semantics require the Sentinel LLM to produce a single-line canonical token (e.g., `APPROVE:<verdict_id>`) and a deterministic checksum; the Orchestrator verifies the checksum against a stored system prompt artifact.
- Any non-exact match (missing token, extra text, parsing failure) is treated as `Refuse`.

3) Human Visibility & Operator Controls
--------------------------------------
- Mandatory pre-execution human confirmation: operator UI must show proposed action, diffs, risk level, and require an explicit two-step confirmation (e.g., operator password + hardware token or 2FA).
- Live progress and telemetry: while the action executes, the UI must display live progress and allow the operator to abort.
- Post-execution: operator must confirm result and sign-off; if operator is unavailable, the system reverts to `Refuse` semantics.

4) Kill-switch (hard, out-of-band)
---------------------------------
- There must be a kill-switch that meets all of these requirements:
  - Operable by at least one human operator outside the LLM and orchestrator process (e.g., systemd service disable, firewall rule, hardware relay, operator console with offline auth).
  - Immediate effect: stop running action(s) and prevent any new action from starting.
  - Independently testable and documented with runbook steps.
- Implementation options (choose >=1): OS-level supervisor stop (systemctl stop aura-backend), network-level block (iptables/block port), or a physical E-stop tied into the operator console.

5) Proof of Execution / Forensics
---------------------------------
- Required artifacts for each execution:
  - Append-only proposal record (persisted to RocksDB) with content hash and MMR proof.
  - Sentinel verdict record with `verdict_id`, signed by an operator key or HSM where possible.
  - Execution transcript logs with timestamps, process IDs, and checksums for all outputs.
  - Post-state snapshot and verification (e.g., artifact digest, session snapshot) stored in the audit store.
- All artifacts must be exportable for independent verification and forensics.

6) Rollback & Compensating Actions
-----------------------------------
- Define rollback narrative for partial success and failure cases:
  - If action is fully idempotent: re-apply idempotency token to re-assert state.
  - If action created an auditable artifact: create a compensating `RevokeArtifact` proposal that goes through Sentinel and operator approval.
  - If action touched an external system (not allowed in initial step): manual rollback runbook must exist and be rehearsed.
- Every rollback must itself be a `CouncilMsg` flow (proposal → sentinel → operator sign-off → execution) and produce its own proofs.

7) Acceptance Criteria (pre-enable checks)
------------------------------------------
- All of the following must be satisfied before any merge/PR enabling `ExecutionMode::Live` is accepted:
  1. Unit & integration tests passing with coverage thresholds.
 2. E2E dry-run rehearsals demonstrating the action path and proof collection.
 3. Operator UI implemented, tested, and available for confirmations and aborts.
 4. Kill-switch tested and demonstrably effective.
 5. Audit artifacts export validated and MMR proofs verified by a separate process.
 6. Security review completed (threat model, access control, secrets handling).

8) Testing & Rehearsal Plan
--------------------------
- Phases of rehearsal (no production effects):
  - Unit tests: adapter parsing, registry behavior, verdict loop.
  - Simulation: DryRun end-to-end with fake LLM responses (approve / deny / invalid) producing audit artifacts.
  - Canary rehearsal: run single-action in an isolated environment with operator in the loop; verify kill-switch and rollback.
  - Post-rehearsal audit and sign-off.

9) Access Control, Auditing & Secret Management
----------------------------------------------
- Ensure least-privilege: only named operator roles can initiate Step 5 flows.
- Keys used for signing verdicts or operator confirmations must be stored in an HSM or equivalent and rotated per policy.
- Audit logs are immutable and retained per retention policy; consider offsite archival for high-integrity retention.

10) Operational Runbook (high-level)
-----------------------------------
- Initiation: operator prepares proposal → Orchestrator persists proposal → Sentinel produces `Approve` token → operator UI prompts for confirmation.
- Execution: operator confirms → system executes (single-action) → logs and proofs produced → operator monitors and may abort via kill-switch.
- Post-execution: operator signs off or triggers rollback flow; artifacts exported for audit.

11) Change Control & Sign-off
----------------------------
- Required sign-offs before any code enabling `Live`:
  - Core dev lead
  - Security & infra lead
  - Operations/On-call lead
  - Two independent auditors (one security, one procedural)
- Each sign-off must be recorded as a `CouncilMsg::Decision` entry and persisted.

12) Emergency Abort & Post-Mortem
--------------------------------
- Abort: document immediate abort steps mapping to the kill-switch options; include contacts and escalation chain.
- Post-mortem: if any unexpected behavior occurs, produce a formal post-mortem with timeline, artifacts, and corrective actions before further enablement.

Appendix: Minimal allowed examples (for review only)
--------------------------------------------------
- Allowed initial action candidate (example): `AppendAuditArtifact` — append a string artifact to the audit store only. No external side effects. Idempotent by artifact-id.

Governance note
---------------
This document is the gating artifact for Phase 4 Step 5. No code enabling live execution may be merged until this checklist items 1–7 are satisfied and sign-offs recorded.
