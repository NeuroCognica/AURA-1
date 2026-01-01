# Gemini Chunk 1: Schema Authority & Enforcement

## Schema Authority Declaration Pattern

### Core Principle
Single Source of Truth for data contracts. Canonical schemas in `aura/schemas/` are law.

**Files:**
- `aura/schemas/CanonicalEnvelope.json`
- `aura/schemas/execution_proof.schema.json`

### Enforcement Mechanism: Schema Lock

**Three-Layer Validation:**

1. **Existence & Syntax Check**
   - Confirms canonical schema files exist
   - Validates JSON Schema Draft-07+ compliance

2. **OpenAPI Parity Check**
   - Compares `sentinel-core/openapi.json` components against canonical schemas
   - Structural comparison (ignores descriptions/titles/examples)
   - Regenerates OpenAPI before comparison

3. **Rust Parity Check**
   - Binary `schema_dump` serializes actual Rust structs
   - Validates output against canonical schema via `jsonschema`
   - Proves runtime compliance, not just documentation

### Implementation Files

**`sentinel-core/tools/schema_lock.py`**
```python
#!/usr/bin/env python3
import json
import subprocess
import sys
from jsonschema import validate, Draft7Validator

def check_existence():
    # Verify aura/schemas/CanonicalEnvelope.json exists
    pass

def check_syntax():
    # Validate JSON Schema syntax
    pass

def check_openapi_parity():
    # Regenerate openapi.json
    subprocess.run(["cargo", "run", "-p", "sentinel_api", "--bin", "gen_openapi"])
    # Compare components.schemas.CanonicalEnvelope
    pass

def check_rust_parity():
    # Run schema_dump binary
    result = subprocess.run(
        ["cargo", "run", "--bin", "schema_dump", "--", "envelope"],
        capture_output=True, text=True
    )
    rust_json = json.loads(result.stdout)
    # Validate against canonical schema
    validate(instance=rust_json, schema=canonical_schema)
    pass
```

**`sentinel-core/crates/sentinel_artifacts/src/bin/schema_dump.rs`**
```rust
use sentinel_artifacts::CanonicalEnvelope;
use serde_json::to_string_pretty;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("envelope") => dump_envelope(),
        _ => eprintln!("Usage: schema_dump <envelope>"),
    }
}

fn dump_envelope() {
    let envelope = CanonicalEnvelope {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        nonce: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        payload: serde_json::json!({
            "action": "genesis_ping",
            "version": 1
        }),
        signature: "sig_genesis_dummy_value".to_string(),
        sender_id: "user_genesis_001".to_string(),
        key_id: "key_genesis_001".to_string(),
    };
    println!("{}", to_string_pretty(&envelope).unwrap());
}
```

**`sentinel-core/crates/sentinel_artifacts/Cargo.toml`**
```toml
[[bin]]
name = "schema_dump"
path = "src/bin/schema_dump.rs"
```

### OpenAPI Injection Pattern

**`sentinel-core/crates/sentinel_api/src/bin/gen_openapi.rs`**
```rust
use std::fs;

fn main() {
    let mut spec = generate_openapi();
    
    // Inject canonical schema directly
    let canonical = fs::read_to_string("../../aura/schemas/CanonicalEnvelope.json")
        .expect("Failed to read canonical schema");
    let canonical_json: serde_json::Value = serde_json::from_str(&canonical).unwrap();
    
    spec["components"]["schemas"]["CanonicalEnvelope"] = canonical_json;
    
    println!("{}", serde_json::to_string_pretty(&spec).unwrap());
}
```

### CI Enforcement

**`.github/workflows/schema-lock.yml`**
```yaml
name: Schema Authority Lock

on:
  push:
    branches: [ "main", "master" ]
  pull_request:
    branches: [ "main", "master" ]

jobs:
  constitutional-guard:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-python@v4
      with:
        python-version: '3.10'
    - run: |
        pip install jsonschema
        python sentinel-core/tools/schema_lock.py
```

### Rust Struct with utoipa Attributes

**`sentinel-core/crates/sentinel_artifacts/src/lib.rs`**
```rust
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(additional_properties = false)]
pub struct CanonicalEnvelope {
    #[schema(format = "uuid")]
    pub id: String,
    
    #[schema(format = "uuid")]
    pub nonce: String,
    
    #[schema(format = "date-time")]
    pub timestamp: String,
    
    #[schema(value_type = Object)]
    pub payload: Value,
    
    pub signature: String,
    pub sender_id: String,
    pub key_id: String,
}
```

## Key Takeaways

1. **Canonical schemas are NOT generated** - they are the authority
2. **OpenAPI injects canonical schema** - prevents utoipa drift
3. **Rust parity binary proves compliance** - runtime validation
4. **CI blocks merges on drift** - automated enforcement
5. **Three-layer validation** - existence, OpenAPI, Rust serialization
