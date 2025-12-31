you will spend no more than 275 manus credits to complete this task. This is a self-contained open-source only project that must be completely offline. You will expand on the processes listed below in full detail. if you need to rearrange order of tasks for efficiency then do that. your purpose is to understand the whole goal and refine and purify while expanding the process checklist. 

## I. WHAT HAS BEEN BUILT SO FAR (FACTUAL STATE)

### 1. Core Philosophy (Locked)

* AURA is a **sovereign, local-first, constitutional AI operating system**, not a chatbot.
* **Sentinel-first, Sentinel-last** development doctrine.
* Authority is explicit, replayable, persisted, and never inferred.
* Separation of concerns is enforced:

  * Authority ≠ language
  * Chat ≠ artifacts
  * UI ≠ cognition

---

### 2. Backend: Authority Spine (IMPLEMENTED)

**Language / Stack**

* Rust
* Axum + Tokio
* Ollama (local LLM runtime)
* RocksDB (append-only, immutable semantics)

**Key Capabilities**

* Typed `CouncilEnvelope` and `CouncilMsg`
* Persist-first authority events
* Replayable `/ws/council` channel with:

  * `Hello`
  * `last_ack`
  * ordered replay
  * `replay_done`
* Non-authoritative `/ws/ai` token stream
* Generation cancellation on deny / interrupt
* No resumable canceled generations
* CI matrix enforcing regression immunity across feature combinations

**Status**

* Authority Spine v1 complete and merged
* CI Phase 1 (Regression Immunity) effectively complete
* Backend is **constitutionally stable**

---

### 3. Governance & Build Process (LOCKED)

**Council Build Loop**

* Manager (this chat): orchestration, sequencing, Copilot instructions
* Backend Builder: Rust, authority, persistence, orchestration
* Frontend Builder: UI, Electron, state, rendering
* Human operator syncs all three

This is now the **official production flow**.

---

## II. WHAT AURA IS EVOLVING INTO (SYSTEM VIEW)

AURA is becoming a **Personal Cognitive Operating System**, consisting of:

* A **Council Orchestrator** (backend)
* Multiple **Archetypal Agents**, each with real duties
* A **Workbench UI** that routes intent, not just messages
* A **Forever Log** of artifacts, plans, notes, and decisions
* A **Life OS layer** (health, work, planning)
* A **Developer OS layer** (projects, code, systems)

---

## III. ARCHETYPES (ALL ACTIVE BY DESIGN)

All archetypes are present **from the beginning**; no feature-flagging them away.

### Core Archetypes & Roles

* **Sentinel** — authority, safety, sanity, law enforcement
* **Architect** — synthesis, project planning, task decomposition
* **Oracle** — pattern prediction, outcome forecasting
* **Explorer** — governed web research
* **Mentor** — career planning, life strategy, education path
* **Empath** — health, mood, routines, diary
* **Jester** — Rule 14 enforcement via humor and interruption

**Key Rule**

* No archetype acts without Sentinel approval.
* Archetype selection is an **authority event**.

---

## IV. UI CONCEPT (FROM YOUR SKETCHES, EXPANDED)

### AURA MAIN COCKPIT (Electron Fullscreen)

#### Layout Regions

* **Top Bar**

  * AURA branding
  * About / AIBOR / Credits / Logo

* **Left Column**

  * Archetype selector (dropdown)
  * Council Mode toggle
  * Chat window (conversation only)
  * Input bar + send
  * Chat history (scroll)

* **Center Column**

  * Blender / Three.js rendered viewport (static initially)
  * System Monitor

    * CPU load/temp
    * GPU VRAM/temp/load
    * Memory
    * Service status
  * Calendar widget

* **Right Column**

  * Archetype Profile panel
  * Archetype image/icon
  * **Custom Notepad**

    * code + text
    * submit → immutable artifact (Forever Log)

#### Global Controls

* TTS toggle
* STT toggle

---

### CUSTOM LAUNCHER (Electron Window / Modal)

**Purpose**

* Process orchestration only (not authority)

**Capabilities**

* Launch Ollama
* Launch Backend
* Launch Frontend
* Red/green status lights
* Health polling
* Launch All
* Kill All

---

## V. KEY FUNCTIONAL SUBSYSTEMS (NOT YET BUILT)

### 1. Council Orchestrator (Backend – Missing Core)

* Classifies user intent
* Routes to correct archetype(s)
* Selects model per task (Mistral, DeepSeek, etc.)
* Manages multi-archetype flows
* Emits structured plans, predictions, artifacts
* Writes to correct surfaces (chat, notepad, calendar, life ops)

This is the **next true backend milestone**.

---

### 2. Notepad / Artifact System

* Open-source editor (Monaco / CodeMirror)
* Separate from chat
* Read/write by agents with Sentinel approval
* Submit = append immutable artifact
* Used for:

  * code
  * plans
  * specs
  * outputs

---

### 3. Explorer (Sentinel-Safe Web)

* Governed retrieval, not free browsing
* Allowlist-first
* Logged URLs + content hashes
* Later: Electron BrowserView with strict policy

---

### 4. Calendar / Scheduling

* Natural language → event creation
* Local-first persistence
* Sentinel-approved writes
* UI calendar reflects backend truth

---

### 5. Voice System (TTS / STT)

* Toggleable
* Unique voice per archetype
* STT → Sentinel → routing
* TTS only on approved outputs

---

### 6. Theme + Sensory Engine (Rule 14)

* Each archetype triggers:

  * full theme shift
  * font changes (open source)
  * glassification
  * sound bed
  * voice change
* Implemented as an **Electron-wide event**

---

## VI. ELECTRON + FRONTEND STACK PLAN (OUTLINE)

### Runtime

* **Electron**

  * Fullscreen kiosk-style cockpit
  * Multi-window (main UI + launcher)

### Frontend

* **TypeScript**
* **React**
* **TailwindCSS** (rapid theming + variants)
* **Redux Toolkit / Zustand** (authority + UI state)
* **Three.js** (or Blender-exported assets via Three.js)
* WebSockets for `/ws/council` and `/ws/ai`

### Styling

* Archetype-driven theme configs
* CSS variables + Tailwind themes
* Glassmorphism (backdrop-filter, layers)

---

## VII. DATA & LOGGING PHILOSOPHY

* Chat: ephemeral, replayable
* Notes: immutable artifacts
* Plans: structured, versioned
* Life logs: append-only
* Career logs: append-only
* Everything auditable
* Nothing silently overwritten

---

## VIII. WHERE WE GO NEXT (HIGH LEVEL)

### Immediate Next Phase (After This Outline)

* Finalize Council Orchestrator architecture
* Define archetype I/O contracts
* Define Artifact / Forever Log schema
* Define Electron app skeleton

### After That

* Frontend shell + launcher
* Orchestrator stub
* Archetype theme engine
* Notepad integration
* Calendar integration

---

## IX. STATUS SUMMARY

* Authority Spine: ✅ COMPLETE
* Build Process: ✅ LOCKED
* UI Concept: ✅ DEFINED
* Archetypes: ✅ FULL SCOPE ACKNOWLEDGED
* Execution Tasks: ❌ NOT YET LOCKED (intentionally)

---

below is a .md checklist file that is currently being used by the project builders. you must integrate the new plan with the steps from below that are unfinished. we are proceeding under the new flow standard listed above. you will spend no more than 275 manus credits to complete this task. 