AURA-1
=======

Backend-authoritative Rust workspace with a stubbed Three.js frontend target.

Primary stacks
- Backend (authority): Rust, Axum/Tokio, optional RocksDB/MMR/Tantivy (gated by features).
- Frontend (client-only): TypeScript/JavaScript, Three.js target (stubbed build).

Workspace layout
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

See `STATUS_REPORT.md` for more details and next steps.
    ws.on_upgrade(move |socket| handle_ws(socket, addr))
}
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
