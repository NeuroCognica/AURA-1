# Workbench Launcher Setup - Checkpoint

**Date:** 2026-01-01  
**Status:** ✅ Working  
**Milestone:** Workbench desktop UI successfully integrated with Python launcher

## Overview

Successfully integrated the AURA Workbench (Electron + React desktop UI) with the Python launcher system. The workbench provides a desktop control interface for interacting with AURA archetypes, monitoring system status, and managing AI conversations.

## Architecture Clarification

AURA-1 consists of **three distinct client applications**:

1. **Backend** (`/backend/`) — Rust authority server (port 8080)
   - Axum/Tokio HTTP + WebSocket server
   - RocksDB persistence with MMR integrity
   - Tantivy full-text search
   - AI orchestration (Ollama, Whisper, TTS)

2. **Workbench** (`/workbench/`) — Desktop control interface (Electron)
   - React + TypeScript + Tailwind UI
   - Archetype selector (Architect, Sentinel, Oracle, etc.)
   - Chat panel for AI conversations
   - System monitor, calendar, notepad
   - Three.js viewport placeholder (center column)
   - Runs on developer's desktop via Electron

3. **Frontend** (`/frontend/`) — iOS-compatible VR/AR web client
   - Three.js immersive 3D environment
   - WebGL 2 compatible (iOS Safari)
   - Cardboard-style stereo rendering
   - Separate from workbench (deployed to web)

## Changes Made

### 1. Launcher Configuration (`launcher/launcher.config.json`)

Updated the Python launcher to start workbench instead of the old launcher control panel:

- **vite service**: Changed from `frontend/` (Three.js stub) to `workbench/` (React UI)
  - Port: 5173
  - Serves the workbench React application
  
- **electron service**: Changed from old launcher to workbench Electron wrapper
  - Command: `npm run electron:launch`
  - Working directory: `C:/AURA-1/workbench`

### 2. Electron Main Process (`workbench/electron/main.js`)

**Fixed:**
- Removed duplicate `launcher:status` IPC handler (was registered twice)
- Added automatic DevTools opening for debugging
- Environment variable handling for dev server URL

**Key configuration:**
```javascript
if (process.env.VITE_DEV_SERVER_URL) {
  mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL)
  mainWindow.webContents.openDevTools()
}
```

### 3. State Machine (`launcher/state_machine.py`)

**Updated environment variables:**
- Changed from `AURA_FRONTEND_URL` to `VITE_DEV_SERVER_URL` (matches workbench expectations)
- Added `NODE_ENV: 'development'` flag
- Reduced frontend wait timeout from 10s to 2s (faster startup)
- Changed hostname resolution from `127.0.0.1` to `localhost` (handles IPv6)

**Environment passed to Electron:**
```python
env = {'VITE_DEV_SERVER_URL': url, 'NODE_ENV': 'development'}
```

### 4. Vite Configuration (`workbench/vite.config.ts`)

**Network binding:**
```typescript
server: { 
  port: 5173,
  host: '127.0.0.1',
  strictPort: false
}
```

Forces IPv4 binding (prevents IPv6-only binding issues on Windows).

### 5. Workbench Package Scripts (`workbench/package.json`)

**Added:**
```json
"electron:launch": "electron ."
```

This allows the launcher to start Electron via `npm run electron:launch`.

## Startup Sequence

The Python launcher (`launcher/launcher.py`) orchestrates services in this order:

1. **Ollama** (port 11434)
   - AI inference engine
   - Health check: TCP connection

2. **Backend** (port 8080)
   - Rust authority server
   - Health check: `GET /health`

3. **Workbench Vite** (port 5173)
   - React dev server
   - Health check: HTTP 200 on `/`
   - Detects port from stdout: `Local: http://localhost:5173/`

4. **Electron**
   - Waits for vite to be healthy
   - Receives `VITE_DEV_SERVER_URL` environment variable
   - Loads workbench UI from dev server

## Service Dependencies

```
ollama (11434)
  ↓
backend (8080) — depends on ollama
  ↓
vite (5173) — depends on backend
  ↓
electron — depends on vite
```

## Known Issues & Solutions

### Issue 1: IPv6 vs IPv4 Binding
**Symptom:** Vite binds to `[::1]:5173` (IPv6) but Electron tries `127.0.0.1:5173` (IPv4)  
**Solution:** Use `localhost` in URLs (resolves to available interface) and configure vite with `host: '127.0.0.1'`

### Issue 2: Stale Vite Process
**Symptom:** Config changes to vite.config.ts don't take effect  
**Solution:** Fully restart the Python launcher (kills and restarts all services)

### Issue 3: Port Conflicts
**Symptom:** Vite starts on 5174 or 5175 instead of 5173  
**Solution:** Python launcher detects port from Vite stdout and passes correct URL to Electron

### Issue 4: Duplicate IPC Handlers
**Symptom:** Electron crashes with "handler already registered" error  
**Solution:** Fixed in `workbench/electron/main.js` (removed duplicate `launcher:status` handler)

## Testing the Setup

### Quick Start
```powershell
cd launcher
python launcher.py
```

Click "START ALL" in the launcher GUI. Services start in sequence, and the workbench window opens automatically.

### Manual Verification
```powershell
# Check Vite is running
curl http://localhost:5173

# Check backend health
curl http://localhost:8080/health

# Check Ollama
curl http://localhost:11434/api/tags
```

### Expected Behavior
1. Python launcher GUI shows all services as "HEALTHY" (green)
2. Electron window opens fullscreen with workbench UI
3. Workbench shows:
   - Archetype dropdown (left column)
   - Chat panel and history
   - Three.js viewport placeholder (center)
   - System monitor (CPU, memory, WS status)
   - Calendar and notepad panels (right)

## Development Workflow

### Option 1: Python Launcher (Recommended)
```powershell
cd launcher
python launcher.py
```
Starts entire stack with one button click.

### Option 2: Manual (for Workbench-only Development)
```powershell
# Terminal 1: Backend
cd backend
cargo run --features "persistence search tls"

# Terminal 2: Workbench Vite
cd workbench
npm run dev

# Terminal 3: Electron
cd workbench
npm run electron:launch
```

## File Summary

**Modified:**
- `launcher/launcher.config.json` — Service definitions
- `launcher/state_machine.py` — Environment variables and timing
- `workbench/electron/main.js` — IPC handlers, dev tools
- `workbench/vite.config.ts` — Network binding
- `workbench/package.json` — Added electron:launch script
- `README.md` — Added workbench documentation

**Created:**
- `WORKBENCH_LAUNCHER_SETUP.md` — This document

## Next Steps

### Immediate (Post-Checkpoint)
- [ ] Test on fresh clone to verify launcher setup
- [ ] Add workbench screenshot to README
- [ ] Document archetype theme system

### Future Enhancements
- [ ] Wire workbench chat panel to backend `/api/chat` endpoint
- [ ] Implement real-time WebSocket telemetry display in system monitor
- [ ] Add Three.js viewport integration (head tracking visualization)
- [ ] Connect notepad to backend persistence
- [ ] Implement calendar event sync

### Archetype System
The workbench includes a theme system (`/workbench/src/themes/archetypes.tsx`) with predefined archetypes:
- Architect, Empath, Explorer, Jester, Mentor, Oracle, Sentinel

Each archetype has:
- Color palette
- Typography
- Layout emphasis
- Audio cues (planned)

## Git Checkpoint

**Commit message:**
```
feat(workbench): integrate Electron desktop UI with Python launcher

- Configure launcher to start workbench (React/Electron) instead of old launcher UI
- Fix duplicate IPC handler in workbench/electron/main.js
- Update state machine to pass VITE_DEV_SERVER_URL and NODE_ENV to Electron
- Reduce frontend wait timeout from 10s to 2s for faster startup
- Force Vite IPv4 binding to prevent IPv6 connection issues
- Add electron:launch script to workbench package.json
- Update README with workbench architecture and launcher instructions

The workbench provides a desktop control interface for AURA archetypes,
featuring archetype selection, chat panel, system monitor, and Three.js
viewport placeholder.

Closes: Workbench integration milestone
```

**Tag:**
```
v0.5.0-workbench-launcher
```

## Configuration Reference

### launcher.config.json
```json
{
  "services": [
    {
      "name": "backend",
      "cmd": "cargo run --manifest-path ../backend/Cargo.toml --bin aura-backend",
      "cwd": "../backend"
    },
    {
      "name": "vite",
      "cmd": "npm",
      "args": ["run", "dev"],
      "cwd": "C:/AURA-1/workbench"
    },
    {
      "name": "ollama",
      "cmd": "ollama serve"
    },
    {
      "name": "electron",
      "cmd": "npm",
      "args": ["run", "electron:launch"],
      "cwd": "C:/AURA-1/workbench"
    }
  ]
}
```

### Service Health Checks
- **Backend:** HTTP GET `/health` returns 200
- **Vite:** HTTP GET `/` returns 200
- **Ollama:** TCP connection to port 11434
- **Electron:** Process PID lock exists

---

**Status:** ✅ Ready for checkpoint  
**Verified:** 2026-01-01  
**Next Review:** After archetype backend wiring
