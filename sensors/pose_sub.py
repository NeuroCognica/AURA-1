"""
Simple pose subscriber

PowerShell (from project root):
    cd C:\AURA-1\sensors
    aura-sensors\Scripts\activate
    python pose_sub.py
"""

import asyncio
import ssl
import json
import websockets

WS_URL = "wss://127.0.0.1:8443/ws/pose"

sslctx = ssl.create_default_context()
sslctx.check_hostname = False
sslctx.verify_mode = ssl.CERT_NONE  # local mkcert / self-signed

async def main():
    print(f"[POSE-SUB] Connecting to {WS_URL}")
    # Disable automatic pings/timeouts on the subscriber to avoid keepalive races
    async with websockets.connect(WS_URL, ssl=sslctx, max_size=2**20, ping_interval=None, ping_timeout=None) as ws:
        print("[POSE-SUB] Connected. Waiting for pose frames...")
        try:
            async for msg in ws:
                try:
                    data = json.loads(msg)
                    print(json.dumps(data, ensure_ascii=False))
                except Exception:
                    print(msg)
        except websockets.exceptions.ConnectionClosed:
            print("[POSE-SUB] connection closed")

if __name__ == "__main__":
    asyncio.run(main())
