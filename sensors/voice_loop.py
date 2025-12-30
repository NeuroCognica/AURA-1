"""
Voice sidecar: microphone -> VAD -> optional STT -> wake-word detection -> backend websocket

Usage:
  - Install requirements: `pip install -r sensors/requirements.txt`
  - Optionally install `openai-whisper` and `ffmpeg` to enable transcription-based wake-word.
  - Run: `python sensors/voice_loop.py`

Environment variables:
  - BACKEND_WS: WebSocket endpoint for ingest (default: ws://127.0.0.1:8080/ws/voice-ingest)
  - WAKE_WORD: case-insensitive phrase to look for (default: "hey aura")
  - USE_STT: if set to "1", will run Whisper transcription on speech segments to detect wake-word
  - VAD_MODE: webrtcvad aggressiveness 0..3 (default 2)
  - SAMPLE_RATE: audio sample rate (default 16000)
"""
import os
import asyncio
import json
import time
import tempfile
from datetime import datetime

import numpy as np
import sounddevice as sd
import soundfile as sf
import webrtcvad
import websockets

try:
    import whisper
    _WHISPER_AVAILABLE = True
except Exception:
    _WHISPER_AVAILABLE = False


BACKEND_WS = os.environ.get("BACKEND_WS", "ws://127.0.0.1:8080/ws/voice-ingest")
WAKE_WORD = os.environ.get("WAKE_WORD", "hey aura").lower()
USE_STT = os.environ.get("USE_STT", "0") == "1"
VAD_MODE = int(os.environ.get("VAD_MODE", "2"))
SAMPLE_RATE = int(os.environ.get("SAMPLE_RATE", "16000"))
CHANNELS = 1
FRAME_MS = 30  # frame size for VAD


class FrameBuffer:
    def __init__(self):
        self.frames = []

    def add(self, b: bytes):
        self.frames.append(b)

    def clear(self):
        self.frames = []

    def bytes(self):
        return b"".join(self.frames)


def int16_to_bytes(data: np.ndarray) -> bytes:
    return data.tobytes()


def write_wav(tmp_path: str, audio_bytes: bytes, sample_rate: int):
    # audio_bytes is int16 little-endian PCM
    arr = np.frombuffer(audio_bytes, dtype=np.int16)
    sf.write(tmp_path, arr, sample_rate, subtype='PCM_16')


async def send_event(ws_uri: str, payload: dict):
    try:
        async with websockets.connect(ws_uri) as ws:
            await ws.send(json.dumps(payload))
    except Exception as e:
        print(f"[voice_loop] failed to send event: {e}")


def transcribe_with_whisper(model, wav_path: str) -> str:
    try:
        result = model.transcribe(wav_path)
        return result.get("text", "").strip()
    except Exception as e:
        print("[voice_loop] whisper transcription failed:", e)
        return ""


def run_loop():
    vad = webrtcvad.Vad(VAD_MODE)
    frame_samples = int(SAMPLE_RATE * FRAME_MS / 1000)
    bytes_per_frame = frame_samples * 2  # int16

    model = None
    if USE_STT:
        if not _WHISPER_AVAILABLE:
            print("USE_STT=1 but whisper not available. Install openai-whisper to enable STT.")
            return
        print("Loading Whisper model (this may take a while)...")
        model = whisper.load_model("small")

    buff = FrameBuffer()
    in_speech = False
    silence_frames = 0
    max_silence_frames = int(300 / FRAME_MS)  # 300ms of silence to end utterance

    print(f"Starting audio capture @ {SAMPLE_RATE}Hz, frame {FRAME_MS}ms, VAD mode {VAD_MODE}")

    with sd.RawInputStream(samplerate=SAMPLE_RATE, blocksize=frame_samples, dtype='int16', channels=CHANNELS) as stream:
        while True:
            try:
                frame, overflow = stream.read(frame_samples)
                if overflow:
                    print("[voice_loop] input overflow")
                raw = frame.tobytes()
                is_speech = vad.is_speech(raw, SAMPLE_RATE)

                if is_speech:
                    buff.add(raw)
                    in_speech = True
                    silence_frames = 0
                else:
                    if in_speech:
                        silence_frames += 1
                        if silence_frames > max_silence_frames:
                            # end of utterance
                            audio_bytes = buff.bytes()
                            buff.clear()
                            in_speech = False
                            silence_frames = 0
                            # process utterance
                            timestamp = datetime.utcnow().isoformat() + "Z"
                            if USE_STT and model is not None:
                                with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tf:
                                    tmp_wav = tf.name
                                write_wav(tmp_wav, audio_bytes, SAMPLE_RATE)
                                text = transcribe_with_whisper(model, tmp_wav).lower()
                                print(f"[voice_loop] transcript: {text}")
                                if WAKE_WORD in text:
                                    payload = {"type": "wake", "text": text, "timestamp": timestamp}
                                    asyncio.run(send_event(BACKEND_WS, payload))
                                    print("[voice_loop] wake detected (transcript)")
                                else:
                                    print("[voice_loop] no wake phrase in transcript")
                            else:
                                # No STT: use energy-based heuristic — send a speech event
                                payload = {"type": "speech", "timestamp": timestamp}
                                asyncio.run(send_event(BACKEND_WS, payload))
                                print("[voice_loop] speech event sent (no STT)")

            except KeyboardInterrupt:
                print("[voice_loop] interrupted, exiting")
                break
            except Exception as e:
                print("[voice_loop] error:", e)


if __name__ == "__main__":
    print("Voice sidecar starting. BACKEND_WS=", BACKEND_WS)
    print("USE_STT=", USE_STT, "WHISPER_AVAILABLE=", _WHISPER_AVAILABLE)
    run_loop()
