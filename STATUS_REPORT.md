# STATUS REPORT — AURA-1 (2025-12-29)

Summary:

- End-to-end pose pipeline validated: webcam → MediaPipe FaceLandmarker → `head_tracker.py` → backend `/ws/pose-ingest` → broadcast → `/ws/pose` → `pose_sub.py`.
- Backend built with `persistence`, `search`, and `tls` features and runs behind mkcert TLS for local HTTPS testing (port 8443).
- `sensors/models/face_landmarker.task` downloaded and consumed by the head tracker.

Quick run instructions (developer):

1. Create and activate the sensors venv (optional but recommended):

```powershell
python -m venv aura-sensors
.\aura-sensors\Scripts\Activate.ps1
pip install -r sensors/requirements.txt
```

2. Run the backend (from `backend/`):

```powershell
cd backend
cargo run --features "persistence search tls"
```

3. Run the head tracker (from repo root):

```powershell
cd sensors
python head_tracker.py
```

4. Run the pose subscriber (in another shell):

```powershell
cd sensors
python pose_sub.py
```

Notes & next steps:

- Replace placeholder pose math in `head_tracker.py` with `solvePnP` using canonical landmark points for accurate yaw/pitch/roll.
- Optionally tune WebSocket ping intervals on the sender to avoid keepalive timeouts in high-throughput scenarios.
- Add a small operator-facing visual or CLI to show real-time yaw/pitch and allow recenter commands.

Contact: See repository `aura1.md` architecture notes for design rationale and persistence/search details.
