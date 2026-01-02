# AURA / NeuroCognica — FULLY EXPANDED MASTER IMPLEMENTATION CHECKLIST

> **Sovereign, Local-First, Constitutional AI Operating System**
> *Sentinel-first, Sentinel-last development doctrine.*
> **Target Environment**: VS Code + GitHub Copilot (GPT-5 Mini Agents)

---

## I. CORE PHILOSOPHY & AGENT INSTRUCTIONS
*   **Authority Spine**: The Rust backend is the source of truth. AI agents must never bypass it.
*   **Sentinel Approval**: No cognitive action (LLM generation) is valid without a Sentinel verdict.
*   **Local-First**: All tools and libraries must be open-source and capable of running offline.
*   **Agent Advice**: When implementing, use `tracing` for logs and `anyhow` for errors. Always write integration tests.

---

## II. PHASE 1 — REGRESSION IMMUNITY (CI/CD)
*Goal: Ensure the authority spine remains unbreakable during development.*

*   [ ] **CI Pipeline Setup**
    *   [ ] Create `.github/workflows/aura-ci.yml`.
    *   [ ] **Task**: Install Rust, `cmake`, `nasm`.
    *   [ ] **Task**: Run `cargo test --all-features`.
    *   [ ] **Task**: Run `cargo clippy -- -D warnings`.
    *   [ ] **Advice**: Use GitHub Actions cache for `target/` and `~/.cargo/` to speed up runs.
*   [ ] **Constitutional Tests**
    *   [ ] **Task**: Implement `tests/authority_integrity.rs`.
    *   [ ] **Requirement**: Verify that `CouncilMsg` cannot be modified without a corresponding test update.
    *   [ ] **Requirement**: Ensure `RocksDB` persistence is atomic.

---

## III. PHASE 2 — OBSERVABILITY & MONITORING
*Goal: Real-time visibility into system health and authority events.*

*   [ ] **Metrics Backend (Rust)**
    *   [ ] **Library**: `prometheus` + `lazy_static`.
    *   [ ] **Task**: Define `COUNCIL_ENVELOPES_TOTAL` (Counter) and `ACTIVE_WS_CONNECTIONS` (Gauge).
    *   [ ] **Task**: Expose `/metrics` on the Axum server.
*   [ ] **System Monitor (Frontend)**
    *   [ ] **Library**: `sysinfo` (Rust) + `zustand` (React).
    *   [ ] **Task**: Create a `SystemMonitor` component in the Center Column.
    *   [ ] **Advice**: Fetch CPU/GPU/VRAM data via a dedicated WebSocket channel `/ws/system`.
*   [ ] **Local Dashboard**
    *   [ ] **Tool**: `Grafana` (Local Docker).
    *   [ ] **Task**: Create a dashboard JSON for "Authority Flow" and "Resource Usage".

## III. PHASE 3 — COUNCIL ORCHESTRATOR (THE BRAIN)

> **Goal**: Implement the core cognitive engine that routes user intent, manages multi-archetype flows, and enforces the Sentinel's constitutional authority. This phase establishes the separation of concerns between **Authority (AURA-1)** and **Cognition (Orchestrator)**.

**Status (2025-12-31):** Freeze lifted — proceeding at full speed. Milestone `v0.4.4-orchestrator-safe-cognition` (commit `c2dc61ac`) achieved: adapter registry instantiated from validated config, LLM client injected (no globals), Technician adapter wired, and canonical DryRun-only execution enforced. Phase 4 Step 4 is complete; continue with planned workstreams.

### 3.1. Intent Classification and Routing Protocol

The Orchestrator's first duty is to translate unstructured user input into a structured, actionable plan by identifying the user's intent and selecting the appropriate Archetype(s).

#### 3.1.1. Intent Classifier Module
*   [ ] **Classifier LLM Selection**: Use a fast, reliable local model (e.g., **Mistral-7B-Instruct** via Ollama) dedicated solely to classification.
*   [ ] **Strict Output Schema**: Define a rigid JSON schema for the LLM's output to ensure machine-readability.
    ```json
    {
      "primary_archetype": "Architect | Explorer | Oracle | Mentor | Empath | Jester",
      "secondary_archetypes": ["Explorer", "Oracle"], // Optional list for complex tasks
      "task_summary": "A concise, 1-sentence summary of the user's request.",
      "is_authority_query": true | false // True if the message is about AURA's constitution or state
    }
    ```
*   [ ] **Routing Logic Implementation**:
    *   [ ] **Task**: Implement a `classify_intent(user_msg)` function in Rust.
    *   [ ] **Rule**: If `is_authority_query` is `true`, the primary archetype is **Sentinel**, and the Orchestrator bypasses all other routing logic.
    *   [ ] **Rule**: If `primary_archetype` is **Architect**, the Architect takes over the flow management, coordinating any `secondary_archetypes`.

#### 3.1.2. Archetype I/O Contracts
*   [ ] **Define Structured Inputs**: Every archetype must accept a structured input (e.g., JSON or YAML) from the Orchestrator, not raw text.
*   [ ] **Define Structured Outputs**: Every archetype must return a structured output (e.g., `Architect::Plan`, `Explorer::SearchResult`) which the Orchestrator then formats for the UI.
*   [ ] **Advice for Agents**: When implementing an archetype, focus the LLM prompt on generating the structured output first, then the natural language explanation.

### 3.2. Sentinel Verdict Loop (Constitutional Wrapper)

All cognitive actions are non-authoritative until approved by the Sentinel. This loop is the core of the "Sentinel-first" doctrine.

| Step | Component | Action | Council Message Type | Invariant Enforced |
| :--- | :--- | :--- | :--- | :--- |
| **1. Proposal** | Orchestrator | Generates a proposed action (e.g., "Write to Notepad"). | `CouncilMsg::ProposedAction` | Persist before Broadcast |
| **2. Review** | Sentinel | Runs its LLM (Temp 0.0) against `sentinel_system.txt`. | N/A | Determinism beats persuasion |
| **3. Verdict** | Sentinel | Emits a decision based on constitutional rules. | `CouncilMsg::Verdict` | Authority is explicit |
| **4. Execution** | Orchestrator | Executes the action **only if** Verdict is `Approve`. | `CouncilMsg::Action` | Authority > Cognition |
| **5. Refusal** | Orchestrator | If Verdict is `Deny` or `Refuse`, logs the verdict and informs the user via chat. | N/A | Refusal is success |

*   [ ] **Task**: Implement `request_verdict(proposed_action)` in the Orchestrator.
*   [ ] **Task**: Implement the Sentinel's deterministic LLM call, ensuring it only outputs one of the three Verdict states.
*   [ ] **Requirement**: The Sentinel must be configured with `temperature: 0.0` and a strict system prompt that forbids creative output or deviation from its law.

### 3.3. Multi-Archetype Flow and Deliberation Protocol

For complex tasks, the Architect manages the flow, and when conflicts arise, it initiates a deliberation process involving the human operator.

#### 3.3.1. Architect as Flow Manager
*   [ ] **Task Decomposition**: Architect breaks down the `task_summary` into sequential or parallel sub-tasks, each assigned to a primary or secondary archetype.
*   [ ] **Coordination**: Architect sends structured `CouncilMsg::SubTask` messages to the target archetypes via the authority spine.
*   [ ] **State Management**: Architect maintains the state of the overall task, waiting for results from sub-tasks before proceeding.

#### 3.3.2. Deliberation Protocol
*   [ ] **Trigger**: An archetype returns a result that conflicts with the Architect's plan or another archetype's output (e.g., Oracle predicts failure, Jester points out a flaw).
*   [ ] **Protocol**:
    1.  **Architect** summarizes the conflict and the reasoning of the conflicting archetypes.
    2.  **Architect** broadcasts `CouncilMsg::Deliberation` via the authority spine.
    3.  **UI (Frontend)** intercepts this message and presents the summary to the human operator.
    4.  **Human Operator** issues `CouncilMsg::Decision` (e.g., "Proceed with Architect's plan" or "Follow Oracle's warning").
    5.  **Orchestrator** resumes the flow based on the human's decision.
*   [ ] **Invariant**: The system deliberates; the human decides. The Architect cannot override a conflict; it can only manage the process of resolving it.

### 3.4. Model and Configuration Management

*   [ ] **Configuration**: Create a central `orchestrator.json` config file to define:
    *   LLM endpoint URLs (for Ollama).
    *   Model names for each Archetype (e.g., `Sentinel: llama3:8b`, `Architect: deepseek-coder`).
    *   Archetype-specific LLM parameters (`temperature`, `top_p`).
*   [ ] **Advice for Agents**: The Orchestrator must load all model configurations at startup. When calling an archetype, it must dynamically select the correct model and parameters. This allows for easy tuning of archetype personalities.

V. PHASE 4 — WORKBENCH UI & ELECTRON COCKPIT
Goal: A high-performance, immersive interface for the Personal OS.
Electron Skeleton
Stack: Electron + Vite + React + TailwindCSS.
Task: Implement the 3-column layout (Left: Chat, Center: Viewport, Right: Notepad).
Three.js Viewport
Library: @react-three/fiber + @react-three/drei.
Task: Render a static "AURA Core" geometry.
Advice: Use RequestAnimationFrame carefully to keep CPU usage low in Electron.
Custom Launcher
Task: Build a separate Electron window for service management.
Feature: One-click "Launch All" (Ollama + Backend + Frontend).
﻿
VI. PHASE 5 — ARTIFACT SYSTEM (FOREVER LOG)
Goal: Persistent, immutable storage for code, plans, and decisions.
Notepad Component
Library: Monaco Editor (the VS Code engine).
Task: Integrate into the Right Column.
Feature: Syntax highlighting for Rust, TS, and Markdown.
Immutable Storage
Backend: RocksDB (Append-only).
Task: Implement CouncilMsg::ArtifactSubmit.
Requirement: Every submit creates a content hash and a timestamped entry.
﻿
VII. PHASE 6 — LIFE OS & DEVELOPER OS LAYERS
Goal: Practical utility through safe web research and scheduling.
Explorer (Safe Web)
Library: headless_chrome (Rust) or Playwright.
Task: Implement "Governed Retrieval".
Rule: Explorer can only visit allowlisted domains (e.g., docs.rs, github.com).
Calendar System
Library: FullCalendar (React) + chrono (Rust).
Task: Natural language to iCal conversion.
Advice: Use the Architect archetype to decompose "Schedule a meeting" into a structured event.
﻿
## VIII. PHASE 7 — VOICE & SENSORY ENGINE

> **Goal**: Implement a robust, local-first multi-modal interface that reinforces archetype identity through unique voice profiles and dynamic sensory shifts, all governed by the Sentinel.

### 7.1. Speech-to-Text (STT) Pipeline

The STT pipeline is designed to be a non-authoritative input channel that immediately feeds into the Sentinel-governed Orchestrator.

*   [ ] **STT Library Integration**:
    *   [ ] **Tool**: **OpenAI Whisper** (Local via `whisper.cpp` bindings for Rust/Electron).
    *   [ ] **Task**: Implement a low-latency audio capture module in the Electron frontend.
    *   [ ] **Task**: Stream audio chunks to a dedicated Rust service (e.g., a microservice within the Orchestrator) for processing via `whisper.cpp`.
*   [ ] **Sentinel Input Gate**:
    *   [ ] **Task**: The transcribed text must be wrapped as a `CouncilMsg::UserVoiceInput` and sent to the authority spine.
    *   [ ] **Rule**: The Orchestrator treats this message identically to a text input, immediately routing it through the Intent Classifier and the Sentinel Verdict Loop.
    *   [ ] **Advice for Agents**: Ensure the STT service is configured to use a small, fast model (e.g., `tiny.en`) for near real-time transcription, prioritizing speed over absolute accuracy.

### 7.2. Text-to-Speech (TTS) Pipeline

The TTS pipeline is an output channel, meaning it is only activated after a cognitive output has been approved by the Sentinel.

*   [ ] **TTS Library Integration**:
    *   [ ] **Tool**: **Piper** (Fast, local neural TTS).
    *   [ ] **Task**: Integrate the Piper library into the Rust backend or a dedicated TTS microservice.
    *   [ ] **Task**: Implement a function `generate_speech(text, voice_id)` that outputs a raw audio stream.
*   [ ] **Archetype Voice Assignment**:
    *   [ ] **Task**: Define a mapping in `orchestrator.json` that assigns a unique Piper voice ID to each of the seven archetypes.
    *   [ ] **Example Voice Mapping**:
        | Archetype | Piper Voice ID | Voice Characteristics |
        | :--- | :--- | :--- |
        | **Sentinel** | `voice_id_01` | Clear, authoritative, low pitch |
        | **Architect** | `voice_id_02` | Measured, precise, moderate pace |
        | **Jester** | `voice_id_07` | High energy, slightly unpredictable cadence |
        | **Empath** | `voice_id_04` | Soft, warm, empathetic tone |
    *   [ ] **Rule**: TTS generation is only triggered by a `CouncilMsg::ArchetypeOutput` that has been approved by the Sentinel. The `voice_id` is determined by the `source_archetype` field in the message.

### 7.3. Theme and Sensory Engine (Rule 14 Enforcement)

The Sensory Engine is an Electron-wide event system that provides immediate, non-verbal feedback to the user, reinforcing the current archetype's presence and the system's constitutional state.

*   [ ] **Event Trigger**:
    *   [ ] **Task**: Implement an Electron IPC channel that listens for `CouncilMsg::ArchetypeActivation` events from the backend.
    *   [ ] **Task**: The event payload includes the `target_archetype` and a `sensory_config_id`.
*   [ ] **Dynamic Styling (Glassmorphism)**:
    *   [ ] **Task**: Define archetype-specific CSS variables (e.g., `--primary-color`, `--glass-blur-radius`) in the Tailwind theme.
    *   [ ] **Effect**: When Jester is active, the UI shifts to a high-contrast, slightly distorted theme with a playful sound bed, enforcing **Rule 14** (challenging assumptions via humor and interruption).
*   [ ] **Sound Bed Implementation**:
    *   [ ] **Task**: Assign a unique, looping ambient soundscape (sound bed) to each archetype (e.g., Sentinel: low-frequency hum; Explorer: light, curious synth).
    *   [ ] **Tool**: Use the Electron `Audio` API to cross-fade between sound beds upon archetype activation.
*   [ ] **Font Changes**:
    *   [ ] **Task**: Define a set of open-source fonts (e.g., a monospace font for Architect, a serif font for Mentor) and dynamically load them based on the active archetype.

### 7.4. Testing and Invariants

*   [ ] **Testing**:
    *   [ ] **Task**: Create `tests/voice_pipeline.rs` to verify that a voice input correctly generates a `CouncilMsg::UserVoiceInput` and that a Sentinel-approved output correctly triggers the TTS service with the correct voice ID.
    *   [ ] **Task**: Create an Electron integration test to verify that `CouncilMsg::ArchetypeActivation` correctly triggers the theme shift and sound bed change.
*   [ ] **Invariant**: **TTS only on approved outputs.** The Sensory Engine must never be triggered by a non-authoritative or unapproved message. The STT output must always pass through the Sentinel.

﻿
IX. PHASE 8 — SSSD INTEGRATION (THE FUTURE)
Goal: Safety-first control of the Solid-State Spacetime Drive.
SSSD Simulator
Task: Build a Rust crate for drive telemetry simulation.
Sentinel Interlock
Requirement: No SSSD command is executed without a Sentinel::Approve verdict.
Test: Verify emergency shutdown if Sentinel is offline.
﻿
X. FINAL AGENT INSTRUCTIONS
Read the Invariant: Every phase has a rule that cannot be broken.
Check the Log: Always look at RocksDB to verify persistence before assuming success.
Refusal is Success: If the Sentinel blocks an action, the system is working correctly.

## Appendix — Launcher Integration: Audit & Status

**Summary:** I scaffolded a standalone `launcher/` Electron + Vite + React project that provides an operator console (two windows: Launcher and AURA Interface), sequential service startup, health probes, process management, and IPC for control. The launcher is intended as an operator-only tool (no auto-run).

**Files added:**
- [launcher/package.json](launcher/package.json)
- [launcher/vite.config.ts](launcher/vite.config.ts)
- [launcher/tsconfig.json](launcher/tsconfig.json)
- [launcher/launcher.config.json](launcher/launcher.config.json)
- [launcher/electron/tsconfig.electron.json](launcher/electron/tsconfig.electron.json)
- [launcher/electron/config.ts](launcher/electron/config.ts)
- [launcher/electron/services.ts](launcher/electron/services.ts)
- [launcher/electron/preload.ts](launcher/electron/preload.ts)
- [launcher/electron/main.ts](launcher/electron/main.ts)
- [launcher/src/main.tsx](launcher/src/main.tsx)
- [launcher/src/App.tsx](launcher/src/App.tsx)
- [launcher/src/styles.css](launcher/src/styles.css)
- [launcher/src/ui/ServiceRow.tsx](launcher/src/ui/ServiceRow.tsx)
- [launcher/src/ui/StatusLight.tsx](launcher/src/ui/StatusLight.tsx)
- [launcher/src/ui/LogPane.tsx](launcher/src/ui/LogPane.tsx)

**Checklist (launcher-specific)**
- [x] Scaffold Electron + Vite + React project under `launcher/`.
- [x] Implement `ServiceManager` (process spawn, health checks, sequential launch) at [launcher/electron/services.ts](launcher/electron/services.ts).
- [x] Expose safe IPC via [launcher/electron/preload.ts](launcher/electron/preload.ts).
- [x] Implement `main.ts` window orchestration and IPC handlers at [launcher/electron/main.ts](launcher/electron/main.ts).
- [x] Create React UI with status lights, launch/health buttons, and log pane.
- [ ] Add short `README.md` in `launcher/` with run and dev instructions.
- [ ] Add tests/integration for health endpoints and `launchAll` sequential behavior.
- [ ] Wire backend `/health` and `/api/history` endpoints (if not present) to guarantee truthful probes.

**Run verification notes:**
- The launcher is operator-driven. Typical workflow:

```powershell
cd launcher
npm ci
npm run dev        # run the Vite dev server (UI)

# In another terminal (build + electron)
cd launcher
npm run start      # builds and launches Electron (build produces dist-electron/* preload)
```

- The launcher reads `launcher/launcher.config.json`. Update ports/commands there to match your environment (backend port, Ollama binary path, frontend dev port, etc.).

**Outstanding audit items / next actions**
- Ensure the backend exposes a deterministic `/health` endpoint that returns 200 only when ready: implement or verify at [backend/src/main.rs](backend/src/main.rs) or appropriate handler.
- Implement `/api/history?session=last` in the backend to surface persistent history for the frontend check on load.
- Add `launcher/README.md` with the commands above and notes about `VITE_DEV_SERVER_URL` environment variable for dev mode.
- Add integration tests: verify `launchAll` returns green states only when services are actually ready; simulate failing health to ensure red light behavior.
- Decide where to commit built artifacts (they should be ignored); keep `dist/` and backend data out of repository.

If you want, I will add `launcher/README.md`, create the integration test skeletons, and run local verification steps (npm install, build) — say which you'd like me to do next.