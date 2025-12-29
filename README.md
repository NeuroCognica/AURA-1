AURA-1
=======

Repository snapshot and agent rules

- Primary language: Rust (backend authority)
- Frontend: TypeScript/JavaScript (Three.js client only)

Build / Run

```bash
cargo build --release
cargo run
```

Service layout (expected)

 - /backend/        # Rust crate (authority)
 - /frontend/       # Three.js client
 - /assets/         # Models, textures, audio
 - /data/           # RocksDB, Tantivy, logs
 - /scripts/        # launch + cert helpers

Repository rule

Always update this `README.md` and the web-repo README for any change that affects build, run, or developer workflow. Include exact commands, env vars, and quick verification steps.

Sync strategy (web repo <-> local)

Recommended approach: CI-driven sync using GitHub Actions that builds the frontend and pushes the built artifacts to the web repo. This avoids merge drift, keeps the authoritative source in the backend repo, and prevents large binary assets from being checked in.

Why this approach
- Automated: CI builds and validates before pushing, so no manual sync steps.
- Auditable: commits to the web repo are produced by CI, preserving provenance.
- Safe: large assets and local DBs remain in `.gitignore` and are not accidentally committed.

Two common implementations
- Git subtree push from CI: build `frontend/`, then `git subtree push --prefix frontend/build <web-repo> main` to update the web repo branch.
- CI push with PAT: build artifacts and use `git` in the workflow (with a deploy token) to commit the `build/` output to the web repo.

Quick CI checklist
- Add a GitHub Actions workflow that:
	1. Checks out the repo.
	2. Installs Node and builds `/frontend/` (if present).
	3. Verifies artifacts.
	4. Pushes `frontend/build` to the web repo (subtree or direct push with token).

CI: `frontend-sync` workflow (scaffolded)
- Location: `.github/workflows/frontend-sync.yml`.
- Trigger: pushes to `main` that touch `frontend/**` (or manual `workflow_dispatch`).
- Gate: job runs only if a frontend lockfile/package.json exists.
- Secrets required: `WEB_REPO` (e.g., `NeuroCognica/AURA-1-web` or full URL) and `WEB_DEPLOY_PAT` (PAT with `repo` scope) to push built assets; optional `WEB_REPO_BRANCH` (default `main`) and `FRONTEND_BUILD_DIR` (default `frontend/build`).
- Behavior: installs frontend deps, runs `npm run build`, force-pushes the build dir to `WEB_REPO_BRANCH`. If secrets are missing, it skips publish and logs a note.

Local workflow (Codex + VS Code)
- Keep `frontend/` as a working folder; do not commit `frontend/build` or large assets — they are ignored.
- When coding with the Codex VS Code extension, pin to the Rust-backend authority: run `cargo build --release` for verification; run `npm run build` inside `frontend/` to mirror CI output.
- Ensure `frontend/package.json` and a lockfile exist before expecting CI to publish; CI skips if they are absent.
- For manual web pushes (if CI tokens are unavailable), use `git subtree push --prefix frontend/build <web-repo> <branch>` or a short script in `scripts/` that mirrors the CI steps.

