# aura_quiz_service

Standalone quiz microservice (RocksDB-backed) that collects answers and generates Mirrorborn profiles.

Quick start

1. Build

```bash
cargo build -p aura_quiz_service
```

2. Run (defaults)

```bash
export QUIZ_DB_PATH=./quiz_db
export ARTIFACTS_DIR=./artifacts
cargo run -p aura_quiz_service
```

3. Submit answers (example)

```bash
curl -s -X POST http://127.0.0.1:4001/submit_answer -H "Content-Type: application/json" \
  -d '{"session_id":"test-1","probe_id":"q1","answer":"A"}'
```

4. Force generate profile

```bash
curl -s -X POST http://127.0.0.1:4001/generate/test-1
```

Notes

- The service stores answers in RocksDB under the path in `QUIZ_DB_PATH`.
- When the answered count reaches the configured total (240), the server spawns profile generation automatically.
- Generated profiles are written atomically to the `ARTIFACTS_DIR` as `profile_<session_id>.json`.
