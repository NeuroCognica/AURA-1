# Backend Runbook — Persistence feature

This runbook documents canonical local commands and notes for developing and testing the RocksDB + MMR persistence feature in the backend.

## Quick commands

- Build (release):

```bash
cargo build --release
```

- Run (dev) with persistence, search, and TLS features:

```bash
cargo run --features "persistence search tls"
```

- Run tests (persistence):

```bash
cargo test --features persistence
```

- Start detached (PowerShell):

```powershell
Start-Process powershell -ArgumentList '-NoExit','-Command','cd C:/AURA-1/backend; cargo run --features persistence' -WorkingDirectory 'C:/AURA-1/backend'
```

- Health check (example):

```bash
curl http://127.0.0.1:8080/health
# or in PowerShell
Invoke-RestMethod 'http://127.0.0.1:8080/health'
```

## HTTP API (persistence endpoints)

- POST /append_log
	- Body: JSON representing a `LogEntry` (see code for schema). Returns appended `id`.
- GET /log/:id
	- Returns raw log bytes / JSON for the requested sequence id.
- GET /mmr_root
	- Returns the current MMR root (base64).
- GET /prove/:id
	- Returns the leaf and sibling peaks needed to verify inclusion of `:id`.
- POST /snapshot
	- Body: `{ "dest": "data/checkpoint_from_api2" }`
	- Creates a RocksDB checkpoint at the given destination.

Notes: snapshot implementation removes any pre-existing destination directory before creating the checkpoint.

## Data & Git

- The RocksDB data directory is `backend/data/`. It should be ignored in git:

```
backend/data/
```

Ensure the repository `.gitignore` contains the above entry to avoid committing DB files.

## TLS & Local Device Notes (mkcert)

When testing sensors or microphone access from an iPhone over LAN, use locally-trusted TLS:

1. Install mkcert and create certs:

```bash
mkcert -install
mkcert aura.local 192.168.X.X
```

2. Place the generated cert/key under `backend/certs/` (or an agreed path) and configure Axum/axum-server to use them. Follow the `--features tls` example in `aura1.md`.

3. On iOS, install and trust the mkcert rootCA to allow navigator.mediaDevices and DeviceOrientation access.

## Snapshot & Backup

- Snapshots are RocksDB checkpoints (fast, consistent). Example POST to `/snapshot` can be used to export DB state for offline verification.
- The snapshot code removes the destination directory if it exists; keep this in mind when choosing destination paths.

## CI / Native deps

- CI must run `cargo test --features persistence` to validate persistence logic.
- Native RocksDB and any C++ libs (Whisper, Piper) must be available in the CI runner. If tests fail in CI with missing native libs, install the platform-specific packages or adjust the CI image to include them.

## Troubleshooting

- If `rocksdb` reports a LOCK error, ensure no other process is holding the DB. Stop the running backend before attempting commits/removals of `backend/data/`.
- If snapshot fails due to existing dir, the API will remove it; local scripts should avoid pointing snapshots at important directories.

## Next steps (for maintainers)

- Optionally commit this file and push to `main`.
- Run the integration test locally: `cargo test --features persistence`.
- If CI fails, capture logs and verify native deps on the runner.

---

Generated on: 2025-12-29

## Archetype activation route

This backend exposes a transport-agnostic library handler for archetype activation and a thin HTTP route for convenience:

- Library handler: `aura_backend::archetype_api::activate_archetype_handler` — builds the activation payload and publishes council broadcasts.
- HTTP route: `POST /api/archetype/activate` — forwards to the library handler. The route is wired in `backend/src/main.rs` and intentionally performs no additional logic.

Use the route for quick smoke checks; prefer the library API for programmatic activation (CLI, Launcher, or tests).

