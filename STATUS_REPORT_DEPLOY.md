# Deploy checklist (short)

- Install Ollama on the target host and ensure the `ollama` daemon is running and reachable from the backend host (default `http://127.0.0.1:11434`).
- Place required model files or configure `OLLAMA_DEFAULT_MODEL` environment variable to a deployed model name.
- Ensure Rust toolchain and native deps for RocksDB are installed on build host (C/C++ build tools on Windows, `build-essential` on Linux).
- Stop the running backend service, copy a consistent RocksDB snapshot into `data/rocksdb` (use `snapshot` endpoint if supported).
- Build the backend with required features: `cargo build --release --features "persistence search tls"`.
- Deploy binary and run behind systemd/service wrapper; ensure `SYNTHETIC_TELEMETRY` is unset in production.
- Verify `/health` and `/metrics`; verify Ollama model via `OLLAMA_URL` and `OLLAMA_DEFAULT_MODEL` env vars.
- Optional: run database proofs with `/prove/:id` and compare against backed-up `mmr_root`.
