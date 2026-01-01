AURA-1
=======

Backend-authoritative Rust workspace with a stubbed Three.js frontend target.

[![Authority CI](https://github.com/NeuroCognica/AURA-1/actions/workflows/authority-spine-ci.yml/badge.svg)](https://github.com/NeuroCognica/AURA-1/actions/workflows/authority-spine-ci.yml)

Primary stacks
- Backend (authority): Rust, Axum/Tokio, optional RocksDB/MMR/Tantivy (gated by features).
- Frontend (client-only): TypeScript/JavaScript, Three.js target (stubbed build).

- Workspace layout
- `/backend/` — Rust crate (`aura-backend`) with Axum server + optional persistence/search modules.
- `/frontend/` — Stub web client; `npm run build` copies `public/index.html` into `frontend/build`.
- `/scripts/` — Ops helpers (`deploy_web.sh` for subtree pushes).
- `/data/`, `/assets/` — ignored; hold RocksDB/Tantivy and large assets locally.

Backend quickstart
```bash
cargo build --release           # build backend only
cargo run                       # dev run on :8080
curl -v http://localhost:8080/health   # smoke check
```

Important: to enable persistence/search/tls features use the feature flags shown below. On some platforms native build tooling (cmake, a C/C++ compiler) is required for RocksDB/whisper native crates.

Features & Run examples
- Enable persistence + search + TLS (local dev with mkcert):

```bash
cargo run --features "persistence search tls"
```

- Run tests with persistence/search enabled:

```bash
cargo test --features "persistence search tls" -- --nocapture
```

Backend features (opt-in)
- `persistence`: enable RocksDB + BLAKE3 + Merkle Mountain Range scaffold.
- `search`: enable Tantivy schema/index scaffold.
- `tls`: enable axum-server TLS.
Example: `cargo run --features "persistence search tls"`.

Frontend quickstart (stub build for CI)
```bash
cd frontend
npm install
npm run build   # outputs to frontend/build
```

Sync / deploy (web repo)
- CI workflow: `.github/workflows/frontend-sync.yml` (trigger: push to `main` touching `frontend/**`).
- Secrets: `WEB_REPO` (e.g., `NeuroCognica/AURA-1-web` or full URL) and `WEB_DEPLOY_PAT` (`repo` scope); optional `WEB_REPO_BRANCH` (default `main`), `FRONTEND_BUILD_DIR` (default `frontend/build`).
- Behavior: installs deps, runs `npm run build`, force-pushes build dir to the web repo branch. If secrets are absent, it builds/notes and skips publish.
- Manual fallback: `WEB_REPO=git@github.com:NeuroCognica/AURA-1-web.git ./scripts/deploy_web.sh` after `npm run build`.

Repository rule
- Always update this `README.md` and the web-repo README for any change that affects build, run, or developer workflow.

Implementation roadmap (3 sprints)
1) Telemetry core: harden Axum WS for head pose/audio channels; add message schema + drop-old policy; add TLS (axum-server).
2) Persistence + integrity: wire RocksDB column families, append-only log writer, MMR root tracking, integrity endpoint.
3) Retrieval + inference: add Tantivy indexer (text + embeddings), Ollama/Whisper orchestration, RAG fetch path feeding TTS pipeline.

Actionable backlog (initial)

Focused code snippets (see `backend/src/main.rs`)

Current Status (2025-12-29):

- Pose pipeline validated end-to-end. See `STATUS_REPORT.md` for run instructions and notes.
- Backend built with `persistence`, `search`, and `tls` features; TLS configured for local testing with mkcert-generated certs.

Milestone: v0.4.4-orchestrator-safe-cognition

- Commit: c2dc61ac
- Summary: Completed Phase 4 Step 4 — adapter registry, config-driven LLM wiring, Technician adapter, and a canonical DryRun-only execution chokepoint. Adapters are constructed from validated `orchestrator.json` and the LLM client is dependency-injected (no global). Missing archetypes fail startup (fail-fast).
- CI status: Workspace tests, orchestrator no-default-features, backend feature-matrix, release build, and frontend build all passed locally. Tag `v0.4.4-orchestrator-safe-cognition` pushed to origin.

Note: The system remains in DryRun-only mode; `ExecutionMode::Live` is intentionally disabled until Phase 4 Step 5 authorization and safety checks are complete.

Quick sensor run commands (developer):

```powershell
# Activate sensors venv (optional)
python -m venv aura-sensors
.\aura-sensors\Scripts\Activate.ps1
pip install -r sensors/requirements.txt

# Start backend
cd backend
cargo run --features "persistence search tls"

# In separate shells from repo root:
cd sensors; python head_tracker.py
cd sensors; python pose_sub.py
```

## Workbench Launcher (Desktop Control Interface)

The **workbench** is an Electron-based desktop UI for interacting with AURA archetypes, monitoring system status, and managing conversations. The **Python launcher** orchestrates the full stack startup sequence.

### Quick Start (Python Launcher)

```powershell
cd launcher
python launcher.py
```

The launcher will start services in sequence:
1. **Ollama** (port 11434) - AI inference engine
2. **Backend** (port 8080) - Rust authority server
3. **Workbench Vite** (port 5173) - React dev server
4. **Electron** - Desktop window loads workbench UI

### Architecture: Three Distinct Components

- **Backend** (`/backend/`) — Rust authority server (Axum, RocksDB, Tantivy, AI orchestration)
- **Workbench** (`/workbench/`) — Electron desktop UI (React, Tailwind, archetype selector, chat interface)
- **Frontend** (`/frontend/`) — iOS-compatible Three.js web client (separate from workbench, for VR/AR cockpit)

**Note:** The workbench is the desktop control interface; the frontend is the mobile/web 3D client. These are separate applications.

### Workbench Configuration

The launcher configuration is in `launcher/launcher.config.json`. Key services:
- `backend`: Cargo run with persistence/search/tls features
- `vite`: Workbench dev server (React UI)
- `ollama`: AI inference engine
- `electron`: Desktop window wrapper

### Manual Workbench Development

To run workbench independently (without Python launcher):

```powershell
cd workbench
npm install
npm run dev              # Terminal 1: Vite dev server
npm run electron:launch  # Terminal 2: Electron window
```

See `STATUS_REPORT.md` for more details and next steps.
    ws.on_upgrade(move |socket| handle_ws(socket, addr))
}

Key HTTP & WebSocket endpoints
- Health: `GET /health`
- Metrics: `GET /metrics`
- Archetypes: `GET /api/archetypes` — returns loaded archetype JSON profiles.
- Chat (AI): `POST /api/chat` — main chat ingress (persistence feature required).
- Appeal: `POST /api/appeal` — record an appeal against an AI/Sentinel decision (persistence required).
- Debug session (persistence only): `GET /debug/session/:id` and `GET /debug/session_ext/:id`.
- WebSocket ingest endpoints: `GET /ws/pose-ingest`, `GET /ws/voice-ingest`.
- WebSocket client subscriptions: `GET /ws/pose`, `GET /ws/voice`, `GET /ws/ai`.

Sentinel & Governance
- The codebase contains a first-class `Sentinel` component (see `backend/src/sentinel.rs`). Behavioral notes:
    - Deterministic pre-check (`sentinel_evaluate`) runs in Rust before making any LLM calls.
    - `Sentinel` may `Allow`, `AllowWithWarning`, `RequireConsent`, or `Deny` an action.
    - If `RequireConsent` or a speech intervention is needed, a constrained non-streaming Ollama call is used to produce the message.
    - All Sentinel interventions are appended to the RocksDB append-only log and broadcast to clients.

Appeals
- Appeals are recorded to the append-only log via `POST /api/appeal` (body: `session_id`, optional `user_id`, `reason`, optional `payload`). Appeals are intended for later council/case review and audit.

Troubleshooting & common issues
- Release build can fail if system-level build tools are missing (e.g., `cmake`, `clang`/`gcc`) required by native crates. Install build tools or use a dev container with the required toolchain.
- If TLS fails to start, verify `certs/cert.pem` and `certs/key.pem` or use the mkcert workflow described in `aura1.md`.

Contribution & coding guidelines
- The Rust backend is authoritative: keep business logic, persistence, and AI orchestration in `backend/`.
- Frontend is client-only; do not move server-side logic to the frontend.
- When changing persistence layout or MMR behavior, include migration notes and tests.

Where to go next
- See `aura1.md` for the architecture report and `STATUS_REPORT.md` for operational run instructions and sensor wiring.

CI guarantees
- **CI coverage:** The repository enforces integration tests for the replay semantics and persistence on pushes to `main` via GitHub Actions.
- **Windows persistence checks:** The CI matrix includes Windows persistence runs; failures should be investigated via the workflow logs and reproduced locally.

If you modify persistence or replay code, add tests and ensure the `ws_replay_integration` test remains green in CI.

```
- RocksDB + MMR scaffold (feature `persistence`):
```rust
pub fn append_log(&self, key: &[u8], value: &[u8]) -> anyhow::Result<()> {
    self.db.put(key, value)?;
    Ok(())
}
pub fn push(&mut self, leaf: &[u8]) -> anyhow::Result<Hash> {
    let digest = blake3::hash(leaf);
    self.inner.push(digest.as_bytes().to_vec())?;
    Ok(digest)
}
```
- Tantivy index example (feature `search`):
```rust
let schema = build_schema();
let index = Index::create_in_dir(temp_dir, schema.clone())?;
let mut writer = index.writer(50_000_000)?;
writer.add_document(doc!(schema.get_field("content").unwrap() => "hello aura"))?;
writer.commit()?;
```

Focused code references
- Axum WebSocket handler: `backend/src/main.rs` (`/ws/telemetry`).
- RocksDB + MMR scaffold (feature `persistence`): `backend/src/main.rs` module `storage`.
- Tantivy mini example (feature `search`): `backend/src/main.rs` module `search`.
