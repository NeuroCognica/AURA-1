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
