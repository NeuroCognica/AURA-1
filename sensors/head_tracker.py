import cv2
import asyncio
import json
import ssl
import time
import numpy as np
import websockets

from mediapipe.tasks import python
from mediapipe.tasks.python import vision
from mediapipe import Image, ImageFormat

# ==============================
# CONFIG
# ==============================

WS_URL = "wss://127.0.0.1:8443/ws/pose-ingest"
MODEL_PATH = "models/face_landmarker.task"

# ==============================
# MEDIAPIPE SETUP
# ==============================

BaseOptions = python.BaseOptions
FaceLandmarker = vision.FaceLandmarker
FaceLandmarkerOptions = vision.FaceLandmarkerOptions
VisionRunningMode = vision.RunningMode

options = FaceLandmarkerOptions(
    base_options=BaseOptions(model_asset_path=MODEL_PATH),
    running_mode=VisionRunningMode.VIDEO,
    num_faces=1,
)

landmarker = FaceLandmarker.create_from_options(options)

# ==============================
# WEBSOCKET
# ==============================

ssl_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
ssl_ctx.check_hostname = False
ssl_ctx.verify_mode = ssl.CERT_NONE

# ==============================
# MAIN LOOP
# ==============================

async def main():
    print("[HEAD] Opening webcam")
    cap = cv2.VideoCapture(0)

    if not cap.isOpened():
        raise RuntimeError("Webcam failed to open")

    async with websockets.connect(WS_URL, ssl=ssl_ctx) as ws:
        print("[HEAD] Connected to backend")

        frame_id = 0
        start = time.time()

        while True:
            ret, frame = cap.read()
            if not ret:
                break

            h, w, _ = frame.shape
            mp_image = Image(
                image_format=ImageFormat.SRGB,
                data=frame
            )

            timestamp_ms = int((time.time() - start) * 1000)
            result = landmarker.detect_for_video(mp_image, timestamp_ms)

            if result.face_landmarks:
                # Simple center-of-face estimation (placeholder for solvePnP later)
                lm = result.face_landmarks[0]
                xs = [p.x for p in lm]
                ys = [p.y for p in lm]

                cx = float(np.mean(xs))
                cy = float(np.mean(ys))

                yaw = (cx - 0.5) * 2.0
                pitch = (0.5 - cy) * 2.0
                roll = 0.0

                payload = {
                    "ts": time.time(),
                    "frame": frame_id,
                    "yaw": yaw,
                    "pitch": pitch,
                    "roll": roll,
                    "confidence": 1.0,
                }

                await ws.send(json.dumps(payload))

            frame_id += 1
            await asyncio.sleep(0.01)

    cap.release()

if __name__ == "__main__":
    asyncio.run(main())

