**AURA Launcher — Detailed Report**

**Generated:** 2026-01-01

**Summary:**
- The `launcher/` directory contains an Electron + Vite + React control-plane application (the "AURA Launcher").
- Purpose: act as the single authority for starting, stopping, monitoring, and diagnosing the core local services that comprise AURA: `backend` (Rust), `frontend` (UI), and `ollama` (inference).
- The launcher exposes a small operator UI with per-service status lights, Start/Stop controls, and a live log pane.

**Files and Key Artifacts:**
- `package.json` — scripts and build metadata for the launcher (dev, start, build, package).
- `index.html`, `src/*` — Vite + React app that renders the launcher UI (`App.jsx`, `main.jsx`, `style.css`).
- `electron/main.js` — Electron main process that creates the window, resolves dev vs. built assets, and wires IPC handlers.
- `electron/preload.js` — Context-bridging API exposing `window.launcher` to the renderer: `listServices`, `startService`, `stopService`, `onStatus`, `onLog`.
- `electron/services.js` — `ServiceManager` implementation: spawn/kill services, capture stdout/stderr, poll health (HTTP/TCP), buffer logs, emit status/log callbacks.
- `launcher.config.json` — Operator-editable configuration containing service definitions, commands/args, working directories, and health URLs.
- `create_shortcut.ps1` — Helper to create a Desktop shortcut that launches the Electron binary directly (dev flow).
- `LAUNCHER_REPORT.md` — (this document) full detail and runbook.

**How the Launcher Works (runtime flow):**
1. Electron `main.js` resolves the app URL in this order:
   - Probe common Vite dev ports (5173, 5174, 5175).
   - If no dev server is found, attempt to start `npm run dev` automatically (dev-only fallback).
   - If a dev server still isn't available, fall back to `file://.../index.html`.
2. `ServiceManager` is instantiated with `launcher.config.json` and registers per-service state (status, pid, logs).
3. The renderer calls `window.launcher.listServices()` to populate the UI; `onStatus` and `onLog` push updates into UI.
4. Start/Stop actions call `svc-start`/`svc-stop` IPC handlers which in turn call the `ServiceManager` to spawn or kill child processes.
5. `ServiceManager` supports two service command shapes:
   - Legacy single-string commands: `{ "cmd": "ollama serve" }`
   - Explicit command + args: `{ "cmd": "npm", "args": ["run","start"] }` (recommended)
6. Health checks: HTTP (GET + 2s timeout) or TCP connect (1.5s timeout) polled every `healthIntervalSeconds` (default 3s).

**Launcher Desktop Icon (dev flow)**
- Purpose: provide a single-click entrypoint that launches the Electron app directly — no PowerShell or npm wrappers.
- Location created by script: `%USERPROFILE%\\Desktop\\AURA Launcher.lnk`
- Shortcut fields (dev):
  - Target: `C:\AURA-1\launcher\node_modules\electron\dist\electron.exe`
  - Start in: `C:\AURA-1\launcher`
  - Arguments: `.`
  - Description: `AURA Launcher (Electron)`
  - Icon: `launcher/assets/icon.ico` (optional; configured in `create_shortcut.ps1`)
- This shortcut loads the app directory directly; Electron resolves the Vite dev server (or loads the built `index.html`).

**Service Contract and Configuration**
- `launcher/launcher.config.json` defines services. Each service includes:
  - `name` — logical name used in UI (e.g., `backend`, `frontend`, `ollama`).
  - `cmd` — command or executable name (e.g., `npm`, `cargo`, `ollama`).
  - `args` — optional array of arguments when using `cmd + args` shape (recommended for Windows).
  - `cwd` — working directory (absolute or relative to `launcher/`). Use absolute paths on Windows to avoid path drift.
  - `health` — object describing health probe; supported types: `http` (url) or `tcp` (host+port).

**Runbook (developer / operator)**
- Dev setup (one-time):
  1. From repository root: `cd launcher && npm install` (installs electron, vite, react) — required for dev desktop shortcut.
  2. In `C:\AURA-1\frontend`: `npm install` to ensure `npx vite` and other frontend assets are available.
  3. Create the desktop shortcut: `powershell -NoProfile -ExecutionPolicy Bypass -File create_shortcut.ps1` (script produces `%USERPROFILE%\\Desktop\\AURA Launcher.lnk`).

- Daily run (dev flow):
  1. Double-click the `AURA Launcher` desktop icon. Window should open.
  2. In the launcher UI use the Start/Stop buttons for each service.
  3. Watch the Logs pane for startup output. Health lights indicate service readiness.

- Packaging (endgame):
  - Use `electron-builder` to produce a Windows installer (`npm run dist`). Packaged binary avoids any runtime dependency on Node/npm and is the recommended production flow.

**Recent Changes Applied (summary of repository edits performed)**
- Added `launcher/package.json` (minimal dev scripts).
- Implemented `index.html`, `src/*` React UI and styling.
- Added `electron/main.js`, `electron/preload.js`, and `electron/services.js` (ServiceManager).
- Created `launcher/launcher.config.json` with service definitions and health URLs.
- Added or adjusted `frontend/package.json` to include `start: "npx vite"` and `dev: "npx vite"` scripts so `npm run start` works reliably on Windows.
- Reworked `create_shortcut.ps1` to point at the packaged Electron binary under `node_modules/electron/dist/electron.exe` and to write the Desktop `.lnk`.

**Troubleshooting**
- Symptom: Launcher opens but UI is blank.
  - Confirm Vite dev server is running and that Electron loaded the correct URL. In `electron/main.js` the process probes common ports (5173/5174/5175) and falls back to `file://index.html`.
- Symptom: `vite` not recognized when starting frontend.
  - Fix: Use `npx vite` in `frontend/package.json` scripts and ensure `npm install` is run in `frontend/` so `npx` can install or use local bin.
- Symptom: `cargo run` ambiguous binary error.
  - Fix: Use explicit `--bin aura-backend` argument in `launcher.config.json` command.
- Symptom: IPC errors like `No handler registered for 'svc-list'`.
  - Fix: Ensure the Electron main process registers IPC handlers before renderer calls, and/or make `svc-list` idempotent and return an empty array until the manager is ready.

**Security & Safety Notes**
- The `ServiceManager` currently spawns child processes with `shell: true` to permit complex command strings. This is convenient during development but carries injection risk if config is untrusted. For production, prefer explicit `cmd + args` arrays and `shell: false`.
- Logs are retained in memory (capped) and exposed to the renderer via IPC. Avoid exposing sensitive secrets in service stdout.

**Next Recommended Steps**
1. Finalize and test packaged `electron-builder` configuration; produce a signed installer for Windows and create official desktop/start-menu entries.
2. Replace `shell: true` spawns with safe `cmd + args` runs and validate cross-platform behavior.
3. Add integration tests that exercise `launchAll`, stop, and health-failure recovery.
4. Add a simple persistence of last-run service states if desired (append-only audit log per Copilot rules — prefer RocksDB for backend authority in the main repo).

**Contact & Verification**
- I verified the following manually during patching on this machine:
  - The frontend `start` script now launches via `npm run start` and Vite reports ready at `http://localhost:5173/`.
  - The desktop shortcut `%USERPROFILE%\\Desktop\\AURA Launcher.lnk` was created and launches Electron directly (dev flow).

If you want, I can now:
- Produce a packaged installer using `electron-builder` (Option 1), or
- Harden `ServiceManager` to avoid `shell: true` and implement strict `cmd + args` paths for all services.

---
End of report.
