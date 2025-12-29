<!-- Short, focused instructions for AI coding agents working in this workspace -->
# Copilot instructions (repo-specific)

Workspace now includes a Rust backend crate (`/backend`), a stubbed frontend (`/frontend`), CI workflow, and the architecture report (`aura1.md`). Guidance below is strict: the Rust backend is the authority; the frontend is client-only.

1. Quick repo summary
- Layout: `/backend` (aura-backend Axum server), `/frontend` (stub Three.js client, npm build to `frontend/build`), `/scripts` (deploy helper), `.github/workflows/frontend-sync.yml` (CI publish), `aura1.md` (architecture).
- Cargo workspace root: `Cargo.toml` with member `backend/`.

2. Primary language and authority
- Primary language: Rust
- Authority: The Rust backend is the system authority (networking, persistence, AI orchestration, integrity).
- Frontend: TypeScript/JavaScript (Three.js web client) is a client only and must not own state or business logic.

3. Runtime model
- Single Rust process (monolithic binary).
- Async actor-style tasks using Tokio.
- No blocking work on telemetry or audio paths.
- AI inference (Ollama, Whisper, TTS subprocesses) must be isolated from real-time loops and not run synchronously on audio/head-tracking threads.

4. Persistence and integrity rules
- Persistence rules:
  - All logs are append-only.
  - RocksDB is the only permitted datastore; do not introduce SQLite, JSON files, or arbitrary files for core state.
  - Merkle Mountain Range (MMR) is mandatory for integrity-sensitive sequences.
  - No mutable "session state" without an audit trail (append-only events or Merkle-backed records).

5. Frontend guardrails
- Frontend constraints:
  - Must run in iOS Safari / WebKit.
  - WebGL 2 only.
  - No dependency on WebXR.
  - Prefer Three.js for web delivery; do not propose Godot web exports for iOS.
  - Stereo rendering: Cardboard-style split + distortion shader when stereo mode is needed.

6. What to do first (discovery)
- If source appears, locate these expected folders and treat them as authoritative roots:
  - `/backend/`       # Rust crate (authority)
  - `/frontend/`      # Three.js client (client-only)
  - `/assets/`        # Models, textures, audio
  - `/data/`          # RocksDB, Tantivy, logs
  - `/scripts/`       # launch + cert helpers
- If these folders are missing, ask the user: "Is the code in a different repository or path? Provide `cargo` workspace layout or a README."

7. Build / run / test (canonical commands)
- Build release: `cargo build --release`.
- Run dev: `cargo run` (for iterative development only).
- The single binary should launch:
  - Axum HTTPS server
  - WebSocket telemetry
  - STT/TTS subprocess orchestration (Ollama client wrappers)
- CI publishing: `.github/workflows/frontend-sync.yml` builds `frontend/` (when `frontend/package.json` exists) and force-pushes `FRONTEND_BUILD_DIR` (default `frontend/build`) to the web repo using secrets `WEB_REPO`, `WEB_DEPLOY_PAT`, and optional `WEB_REPO_BRANCH`. Keep build artifacts out of this repo.
- Backend features: enable `persistence` for RocksDB+BLAKE3+MMR scaffolding; `search` for Tantivy; `tls` for axum-server TLS. Example: `cargo run --features "persistence search tls"`.
- Frontend stub: `cd frontend && npm install && npm run build` (copies `public/index.html` to `frontend/build`).

8. Editing and pull requests
- Do not add large scaffolding without explicit permission. Propose minimal patches with a short test plan.
- When changing persistence, include migration steps and an integrity review (MMR consequences).

9. CI / debug / test troubleshooting
- Inspect `.github/workflows/*.yml` for build steps before modifying CI.
- When debugging failing tests, run the specific failing `cargo` test locally and report stack traces and backtraces.
- If CI publish skips, check that `frontend/package.json` exists and `WEB_REPO`/`WEB_DEPLOY_PAT` secrets are set; build output must land in `FRONTEND_BUILD_DIR`.

10. When to ask the user (prioritize these questions)
- Where is the application source and which folder is the workspace root?
- Exact `cargo` commands for workspace, custom targets, and any cross-compilation requirements.
- Any pinned versions for Ollama, Whisper, TTS, or other inference components.

11. Safe defaults
- Do not run arbitrary shell commands without explicit permission.
- Do not change data storage layers or introduce new datastore types without approval.
- Do not commit large assets, model files, RocksDB/Tantivy data, or generated frontend builds; `.gitignore` excludes these.

- Repository rule: Always update `README.md` in the project (and the web repo counterpart) for any change that affects build, run, or developer workflow. Document commands, service endpoints, and any environment variables added or changed.

12. Example agent flow for this repo
- Step 1: Confirm primary language and repo layout (see section 6).
- Step 2: If asked to implement features, propose a minimal plan and files to change; include `cargo` tests and a short migration note for persistence changes.

If anything here is unclear, or if your repo layout differs from the expected folders, paste a short tree listing or the `Cargo.toml` workspace so I can refine these instructions.
