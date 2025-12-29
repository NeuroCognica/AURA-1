AURA-1 backend prototype

Quick run (development):

1. Build and run the backend (listens on 0.0.0.0:8080):

```bash
cd backend
cargo run
```

2. Enable synthetic telemetry generator (60Hz) for testing:

```bash
SYNTHETIC_TELEMETRY=1 cargo run
```

3. Toggle slow inference to simulate a slow AI worker:

```
curl http://localhost:8080/toggle_slow_inference
```

4. Enqueue an AI job (quick test):

```
curl -X POST -H "Content-Type: application/json" -d '{"prompt":"hello"}' http://localhost:8080/enqueue_ai
```

5. Metrics endpoint (plain text):

```
curl http://localhost:8080/metrics
```

Notes:
- This is a minimal prototype showing an actor-style decomposition with a latest-value telemetry path, bounded queues, and simulated workers.
- Not all production concerns (TLS, persistent RocksDB, Tantivy indexing) are implemented in this prototype.
