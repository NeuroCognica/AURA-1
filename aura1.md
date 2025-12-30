# **Architectural Migration Report: AURA-1 Cockpit Transition to High-Assurance Rust Infrastructure**

## **1\. Executive Summary**

The AURA-1 Cockpit project represents a paradigm shift in local-first, intelligent conversational interfaces. The objective is to transition from a legacy backend—comprised of loosely coupled Python and Node.js services—to a unified, high-performance infrastructure built on Rust. This migration is driven by stringent requirements for **offline capabilities**, **cryptographic data integrity**, **real-time responsiveness**, and **memory safety**.

The proposed architecture centers on a monolithic Rust binary that orchestrates the entire lifecycle of the application. By leveraging **RocksDB** for high-throughput embedded storage and **Merkle Mountain Ranges (MMR)** for append-only cryptographic verification, the system ensures that every conversational turn is both persistent and tamper-evident. The integration of **Tantivy**, a Rust-native full-text and vector search engine, provides the "searchable" requirement without the overhead of external services like Elasticsearch.

Crucially, the system must function as a "cockpit" interface, implying low-latency telemetry for head tracking and real-time audio streaming. The move to Rust’s **Tokio** asynchronous runtime allows for a sophisticated actor-based concurrency model that prevents the heavy computational loads of AI inference (via **Ollama** and **Whisper**) from blocking the high-frequency telemetry loops required by the **Three.js** frontend on iOS.

This report provides an exhaustive technical analysis and implementation roadmap for this transition. It details the selection of specific Rust crates, the internal data structures required for merging RocksDB with Merkle Trees, the strategies for overcoming iOS WebGL limitations, and the precise networking patterns needed to achieve the "AURA-1" vision.

## ---

**2. Architectural Paradigm Shift: From Interpreted to Systems Programming**

The move from a Python/Node.js stack to Rust is an operational shift, not just a language swap. The cockpit is a soft-real-time system: audio buffers, 60Hz head-tracking quaternions, and AI context tensors must be processed with predictable end-to-end latency. That requirement drives concrete architectural choices below.

Core constraints
- Deterministic latency for telemetry (60Hz) and sub-second TTS feedback.
- Minimize memory churn and heap pauses; avoid GC stalls.
- Keep heavy, variable-time work (inference, large vector math, compaction) off the fast paths.

Concurrency model (recommended)
- Single process, Tokio runtime, actor-style decomposition.
- Use tokio::spawn for long-running/offloaded work and tokio::sync::mpsc channels for message passing between actors.
- Prioritize actors by queue discipline rather than thread affinity: small bounded queues for latency-sensitive actors, larger batches for throughput workers.

Suggested actor/task breakdown
- WebSocket Server — accepts connections, cheap parsing, dispatches messages (High priority).
- Telemetry Processor — decodes and folds head pose into the shared state (High priority).
- Audio Buffer Manager — frames, VAD, and short-term buffering (Medium priority).
- AI Orchestrator — prepares prompts, calls inference engines, handles token streams (Offloaded/blocking).
- Persistence Worker — batched RocksDB writes and MMR updates (Throughput oriented, low priority).

Backpressure & packet semantics
- Treat telemetry as latest-value streams: drop/overwrite older packets when queues back up.
- Use bounded channels with explicit overflow policy (e.g., try_send + counter) to avoid unbounded memory growth.
- For audio, implement a sliding window buffer and a hard cap on queued PCM frames.

Zero-copy and memory strategy
- Accept binary frames as owned byte buffers and pass ownership through channels rather than cloning.
- Use arenas or pooled buffers for frequently allocated buffers to reduce allocator overhead.
- Prefer Bytes / BytesMut for shared binary payloads across async boundaries.

Offloading inference & blocking work
- Run inference and heavy postprocessing in dedicated worker tasks or blocking thread pools (tokio::task::spawn_blocking) to keep the async reactor responsive.
- Serialize only minimal metadata to persistence paths; push large work to background tasks that report completion via channels.

Scheduling, tuning, and observability
- Use small, observable metrics: telemetry latency, queue lengths per actor, VAD decisions/sec, persistence lag.
- Expose health and liveness endpoints and a /metrics Prometheus endpoint.
- Start with conservative queue sizes and increase after measuring real device behavior.

Practical primitives (examples)
- Latency path: accept websocket -> parse text JSON -> telemetry_tx.try_send(pose) -> update latest pose (overwriteable atomic/state).
- Heavy path: on finalizing a log entry, ai_tx.send(Work::IndexAndEmbed(log)) -> AI worker returns embedding -> index via Tantivy and persist via RocksDB batch.

Testing checklist
- Synthetic 60Hz telemetry generator to validate end-to-end latency under load.
- Inject a slow inference worker to verify telemetry continues (backpressure/drop behavior).
- Long-run memory profiler to ensure no unbounded growth or allocation spikes.

Design goal: make the fast paths minimal and deterministic; isolate all variable and expensive operations behind clear actor boundaries so that head-tracking and audio remain stable regardless of what the AI or persistence subsystems are doing.

## ---

**3\. Data Persistence Strategy: Embedded RocksDB**

The requirement for a "continuous conversation log" stored in a specific database mandates the use of **RocksDB**. As an embedded key-value store based on Log-Structured Merge-trees (LSM), RocksDB is optimized for the write-heavy workloads typical of logging systems.

### **3.1 Rust Integration via rust-rocksdb**

We utilize the rust-rocksdb crate, which provides high-level bindings to the underlying C++ library.3 This approach allows the Rust application to embed the database engine directly into its process address space, eliminating the network overhead associated with external databases like PostgreSQL or MongoDB.

### **3.2 Schema Design and Column Families**

To maintain organization and performance, the database is partitioned into **Column Families (CF)**. A Column Family in RocksDB is a logical partition that shares the same Write-Ahead Log (WAL) but maintains its own MemTable and SST files. This isolation allows us to tune compaction strategies for different data types.

#### **3.2.1 The logs Column Family**

This CF stores the primary conversation history.

* **Key Format**: \[u8; 8\] \- A Big-Endian 64-bit integer representing the sequence ID.  
* **Value Format**: Vec\<u8\> \- A compressed binary payload (using bincode or CBOR) containing the conversation metadata, transcript, and speaker ID.

**Rationale for Big-Endian Keys**: RocksDB sorts keys lexicographically. By using Big-Endian encoding for the sequence ID (0x0000000000000001), the keys are stored in numerical order on disk. This makes range queries (e.g., "retrieve the last 50 messages") extremely efficient, as they correspond to a sequential read of the SST files.

#### **3.2.2 The state Column Family**

This CF stores the mutable state of the system, such as the current Merkle Tree root hash, the last processed log ID, and user configuration settings.

* **Key**: UTF-8 String (e.g., "mmr\_root", "last\_log\_id").  
* **Value**: Varies (JSON or raw bytes).

#### **3.2.3 The vectors Column Family (Optional)**

If the architecture dictates storing raw embedding vectors alongside the logs (before indexing them in Tantivy), a separate CF allows for distinct block cache settings. Vectors are large and accessed randomly, whereas logs are typically accessed sequentially. Segregating them prevents vector lookups from thrashing the cache used for log retrieval.

### **3.3 Tuning for the AURA-1 Workload**

The AURA-1 workload is characterized by:

1. **Continuous Writes**: Appending new logs and Merkle nodes.  
2. **Occasional Bursty Reads**: When the user performs a search or the AI requests context.

**Optimization Configuration**:

* **set\_use\_direct\_io\_for\_flush\_and\_compaction(true)**: Bypass the OS page cache for background operations to prevent stalling the main application threads.  
* **Block Cache**: Allocate a dedicated block cache (e.g., 512MB) shared across column families to keep the "hot" recent conversation history in memory.  
* **Compression**: Use **LZ4** for the bottommost levels of the LSM tree. Text logs compress highly effectively, saving significant disk space on the local device without incurring the CPU penalty of Zstd.4

#### **3.4 Run & Test (persistence feature)**

Quick commands to build, run, and test the backend with the `persistence` feature enabled. These are the canonical local steps used during development and CI.

- **Build (release):** `cargo build --release`
- **Run (dev) with persistence:** `cargo run --features "persistence search tls"`
- **Run tests (persistence):** `cargo test --features persistence`
- **Start detached (PowerShell):**

```powershell
Start-Process powershell -ArgumentList '-NoExit','-Command','cd C:/AURA-1/backend; cargo run --features persistence'
```

Notes:
- The `persistence` feature enables RocksDB-backed storage and MMR integrity checks. Ensure native RocksDB dependencies are available on your platform when building.
- Use `mkcert` and TLS when testing sensor or microphone access from remote devices (see section 8.3.2).

## ---

**4\. Cryptographic Integrity: Merkle Mountain Ranges**

The user requirement specifies "Merkle Trees for data persistence and integrity." For a system that functions as a continuous log, a standard balanced Merkle Tree is architecturally unsuitable due to the high cost of rebalancing upon every insertion. The correct data structure is the **Merkle Mountain Range (MMR)**.5

### **4.1 Theoretical Foundation of MMR**

An MMR is a deterministic data structure that represents a list of items (leaves) as a set of perfectly balanced binary trees ("mountains"). As new items are appended, they are either added as a new mountain or merged with existing mountains to form a larger one.

**Advantages for AURA-1**:

* **Append-Only Efficiency**: Inserting a new log entry is an $O(\\log n)$ operation.  
* **Immutability**: Once a node is written, it is never modified. This aligns perfectly with the append-only nature of conversation logs.  
* **Proof Capability**: It enables the generation of compact proofs that a specific log entry exists and that the timeline has not been altered (Proof of History).

### **4.2 Implementation: rs-merkle vs. Custom Implementation**

While crates like rs-merkle 6 and merkle-mountain-range 7 exist, integrating them with RocksDB requires implementing a custom storage backend. Most off-the-shelf implementations default to in-memory Vec storage, which is insufficient for a persistent database.

#### **4.2.1 The RocksDBStore Implementation**

The Rust backend must implement a struct that acts as the bridge between the MMR logic and RocksDB.

Rust

// Conceptual Rust Implementation  
struct RocksDBStore {  
    db: Arc\<DB\>,  
    cf\_nodes: ColumnFamily,  
}

impl MMRStore for RocksDBStore {  
    fn append(&mut self, pos: u64, hashes: &\[H256\]) \-\> Result\<()\> {  
        let mut batch \= WriteBatch::default();  
        for (i, hash) in hashes.iter().enumerate() {  
            // Map MMR position to a RocksDB key  
            let key \= (pos \+ i as u64).to\_be\_bytes();   
            batch.put\_cf(self.cf\_nodes, key, hash);  
        }  
        self.db.write(batch)?;  
        Ok(())  
    }  
}

This code snippet illustrates how the abstract positions of the Merkle nodes are mapped to physical keys in the merkle\_nodes Column Family.

### **4.3 The Integrity Workflow**

1. **Input**: The user speaks a command: "Activate privacy mode."  
2. **Logging**: The text is saved to RocksDB logs.  
3. **Hashing**: The content is hashed using **BLAKE3** (chosen for its speed advantage over SHA-256 in Rust).8  
4. **Accumulation**: The hash is pushed to the MMR. The RocksDBStore writes the new leaf hash and any new internal node hashes generated by the merge.  
5. **Root Update**: The new MMR root is calculated ("bagging the peaks") and stored in the state CF.

### **4.4 Auditing and Verification**

To verify integrity, the system can expose a specialized endpoint. An external auditor (or the frontend) can request a proof for log \#42. The backend retrieves the sibling hashes from RocksDB and constructs the Merkle Path. If the recomputed root matches the stored root, the data is mathematically proven to be unaltered.

## ---

**5\. Search and Retrieval: The Tantivy Engine**

The requirement for a "searchable conversation log" necessitates an index separate from the primary storage. While RocksDB excels at retrieving known keys, it cannot efficiently handle queries like "Find all conversations about 'navigation' from last Tuesday."

### **5.1 Tantivy: The Rust-Native Search Solution**

**Tantivy** is the superior choice for this architecture. It is a port of Lucene's design principles to Rust, offering high performance, memory safety, and—crucially—embedding capabilities.4 Unlike setting up an external Elasticsearch node, Tantivy is a library; its index files live alongside the RocksDB files, managed by the single Rust binary.

### **5.2 Schema Design for Conversational Context**

The search schema must be carefully defined to support both full-text search (FTS) and semantic retrieval.

| Field Name | Type | Options | Purpose |
| :---- | :---- | :---- | :---- |
| log\_id | u64 | \`STORED | FAST\` |
| timestamp | i64 | \`INDEXED | FAST\` |
| speaker | Str | \`STRING | STORED\` |
| content | Text | TEXT | Tokenized keyword search (BM25). |
| embedding | Bytes | FAST | Vector storage for semantic search. |

**Insight**: The STORED attribute in Tantivy means the data is kept in the index. However, since we already store the full JSON in RocksDB, we can selectively disable STORED for the heavy content field in Tantivy to save space, using the log\_id to fetch the full record from RocksDB when needed. This reduces the index size and improves cache locality.

### **5.3 Vector Search and RAG**

The AURA-1 system requires semantic understanding. Tantivy 0.22 introduced experimental support for vector search, and related crates extend this capability.10

#### **5.3.1 The Vector Pipeline**

1. **Embedding**: When a log is finalized, the Rust backend sends the text to **Ollama** (via ollama-rs) to generate a vector using a model like nomic-embed-text.11  
2. **Indexing**: This vector is added to the embedding field in the Tantivy document.  
3. **Querying**: When the user asks a vague question, the system embeds the query and performs a K-Nearest Neighbor (KNN) search in Tantivy.

This enables **Retrieval-Augmented Generation (RAG)**. The retrieved historical logs serve as context for the current LLM prompt, allowing AURA-1 to "remember" past interactions even if they occurred days ago.

#### **5.3.2 Tantivy vs. LanceDB**

While **LanceDB** is a popular modern vector database 12, integrating it would introduce another storage format (Arrow) and dependency set. Tantivy is sufficiently performant for conversational log scales (millions of vectors) and maintains architectural homogeneity (pure Rust ecosystem), which is preferable for a robust, embedded application.

## ---

**6\. The Rust-AI Bridge: Offline Intelligence**

The "offline-first" requirement mandates that all AI processing occurs locally. The Rust backend acts as the orchestrator, binding to high-performance inference engines.

### **6.1 Large Language Model: Ollama Integration**

**Ollama** simplifies the deployment of LLMs (Llama 3, Mistral) by providing a local API. The Rust backend communicates with Ollama using the ollama-rs crate.14

#### **6.1.1 Context Management Strategy**

Ollama's API is stateless regarding session history in its basic generate endpoints. To maintain a continuous persona ("AURA-1"), the Rust backend must manage the **Context Window**.

* **Session Struct**: The backend maintains a Session object containing a rolling window of the conversation.  
* **Token Management**: Before sending a prompt, the system appends relevant retrieved logs (from Tantivy) and the recent chat history.  
* **Persistence**: The context tensor returned by Ollama (which represents the KV cache state) is serialized and stored in RocksDB. This allows the system to be restarted and immediately resume the conversation without re-processing the entire history.16

### **6.2 Speech-to-Text: Whisper via FFI**

For offline STT, **Whisper** (OpenAI) is the standard. We utilize whisper-rs, a safe Rust wrapper around Georgi Gerganov’s whisper.cpp.17

**Implementation Detail**:

* The whisper-rs crate links against the C++ library. This requires the Rust build environment (cargo) to have access to a C++ compiler (clang or gcc) during the build process.  
* **Quantization**: To ensure the "cockpit" remains responsive, we use 4-bit or 5-bit quantized models (ggml-base.en-q5\_1.bin). These trade a negligible amount of accuracy for a significant reduction in memory usage and inference time.

### **6.3 Text-to-Speech: Piper Integration**

**Piper** offers fast, neural TTS suitable for real-time feedback.18 Unlike Whisper, direct Rust bindings are less mature. The robust architectural choice here is to use std::process::Command to spawn Piper as a controlled subprocess.

**Streaming Architecture**:

1. **Token Stream**: Ollama streams tokens to Rust.  
2. **Sentence Buffering**: Rust accumulates tokens until a sentence boundary (., ?, \!) is detected.  
3. **Synthesis**: The complete sentence is piped to Piper's stdin.  
4. **Audio Output**: Piper writes raw PCM audio to stdout, which Rust reads, frames, and sends down the WebSocket to the client.

This "stream-by-sentence" approach minimizes the Time To First Byte (TTFB) latency perceived by the user.

## ---

**7\. Real-Time Networking: The Axum/Tokio Core**

The "cockpit" functionality implies a telemetry loop. The Rust backend must serve the frontend and handle bi-directional real-time data.

### **7.1 Axum as the Web Framework**

**Axum** is chosen for its ergonomics and deep integration with Tokio.19 It allows us to define routes for both standard HTTP (serving the Three.js frontend) and WebSockets (telemetry/audio).

### **7.2 WebSocket Architecture for Head Tracking**

Head tracking requires 60Hz updates. While UDP is theoretically faster, WebSockets are the only viable transport for a browser-based client on iOS (Safari) connecting to a local server.

**Latency Optimization**:

* **TCP\_NODELAY**: The Axum WebSocket upgrade must configure the underlying TCP socket with nodelay=true to disable Nagle's algorithm. This prevents the OS from buffering small JSON packets containing head-tracking coordinates.1  
* **Backpressure Handling**: The backend actor handling head tracking must implement logic to drop old packets. If the AI processing thread lags, we must not queue up 50 outdated head positions; we only care about the *latest* position.

**Protocol Design**:

* **Upstream (Client \-\> Server)**:  
  * Binary Message: \`\` (Microphone Audio).  
  * Text Message: {"type": "pose", "q": \[x, y, z, w\]} (Head Tracking).  
* **Downstream (Server \-\> Client)**:  
  * Binary Message: \`\` (TTS Audio).  
  * Text Message: {"type": "log", "text": "..."} (Transcript).

### **7.3 Service Discovery (mDNS)**

To ensure the "offline" and "local" experience is seamless, the Rust backend should implement an mDNS responder (using the mdns-sd crate). This allows the iPhone to connect to aura.local instead of a fluctuating IP address (e.g., 192.168.1.45), significantly improving usability in a headless/cockpit scenario.

## ---

**8\. Frontend Architecture: AURA-1 Cockpit**

The original requirement mentions "AURA-1 cockpit" with "local hosted web frontend (iPhone/VR)". A critical finding in the research phase dictates a move away from the popular Godot engine for this specific use case.

### **8.1 The Case Against Godot 4 for iOS Web**

Multiple sources confirm that **Godot 4 HTML5 exports are currently broken on iOS**.21 Issues include:

* **SharedArrayBuffer**: iOS Safari has strict security requirements for threading support, often requiring specific HTTP headers that are difficult to manage in a simple local setup.  
* **WebGL 2.0**: Inconsistencies in Apple's WebGL implementation cause frequent black screens and crashes in Godot 4 exports.  
* **Performance**: The overhead of the WASM runtime for a full game engine drains battery rapidly, a critical flaw for a mobile-hosted VR cockpit.

### **8.2 The Solution: Three.js / React Three Fiber**

The frontend will be re-engineered using **Three.js**. This is a JavaScript library that runs natively in the browser's graphics pipeline, offering:

* **Stability**: Proven track record on iOS Safari.  
* **WebXR Support**: Native integration with the WebXR Device API for VR headsets.  
* **Lightweight**: Significantly smaller download size and memory footprint than a Godot WASM blob.

### **8.3 VR Implementation Details**

#### **8.3.1 Stereoscopic Rendering & Barrel Distortion**

To simulate a "cockpit" in a Cardboard-style viewer, the screen must be split, and barrel distortion applied to counteract the lens magnification.

Shader Implementation:  
Snippet 25 provides the exact GLSL code required for the barrel distortion fragment shader:

OpenGL Shading Language

uniform float BarrelPower;  
void main() {  
  vec2 uv \= vUv;  
  vec2 p \= 2.0 \* uv \- 1.0; // Map to \[-1, 1\]  
  float theta \= atan(p.y, p.x);  
  float radius \= length(p);  
  radius \= pow(radius, BarrelPower); // Apply distortion  
  p.x \= radius \* cos(theta);  
  p.y \= radius \* sin(theta);  
  vec2 distortedUV \= 0.5 \* (p \+ 1.0); // Map back to   
  gl\_FragColor \= texture2D(tDiffuse, distortedUV);  
}

This shader is applied via the EffectComposer in Three.js, rendering the scene to an off-screen buffer and then drawing the distorted result to the screen.

#### **8.3.2 Accessing Sensors: The HTTPS Requirement**

iOS Safari **blocks access to the gyroscope and microphone on insecure origins** (HTTP sites that are not strictly localhost). Since the iPhone accesses the backend via the LAN IP (e.g., 192.168.1.10), **HTTPS is mandatory**.

**The mkcert Workflow**:

1. **Generate CA**: Use mkcert on the host machine to create a local Certificate Authority.26  
   * mkcert \-install  
   * mkcert aura.local 192.168.X.X  
2. **Install on iOS**:  
   * AirDrop the rootCA.pem to the iPhone.  
   * Settings \> Profile Downloaded \> Install.  
   * **Crucial Step**: Settings \> General \> About \> Certificate Trust Settings \> Enable "Full Trust" for the root certificate.28  
3. **Rust TLS**: Configure Axum to use axum-server with tls-rustls, loading the aura.local-key.pem and aura.local.pem files.

Only after this "green lock" is achieved will the DeviceOrientation API and navigator.mediaDevices.getUserMedia function correctly.

## ---

**Progress Update (2025-12-29)**

The repository now contains a validated end-to-end pose pipeline and updated sensor tooling. Key outcomes since the architecture draft above:

- `sensors/head_tracker.py`: Rewritten to use MediaPipe Tasks (`FaceLandmarker`) and requires `sensors/models/face_landmarker.task` (downloaded). Streams JSON pose frames to the backend ingest WebSocket at `wss://127.0.0.1:8443/ws/pose-ingest`.
- `sensors/pose_sub.py`: Subscriber hardened to avoid WebSocket ping/pong keepalive race errors (disabled automatic pings, graceful close handling). It receives broadcast pose frames from `wss://127.0.0.1:8443/ws/pose`.
- Backend: built with `persistence`, `search`, and `tls` features; Axum/Tokio server listens on TLS port (8443) with mkcert-generated certs for local HTTPS testing. Telemetry and pose broadcast endpoints are wired and tested.
- Model asset: `sensors/models/face_landmarker.task` downloaded into the project and used by the head tracker.

Observed runtime validation:

- Live JSON pose frames were observed end-to-end: webcam → FaceLandmarker → `head_tracker.py` → `/ws/pose-ingest` → backend broadcast → `/ws/pose` → `pose_sub.py`.
- Previously observed errors (MediaPipe namespace mismatches and a script SyntaxError) were resolved by migrating to the Tasks API and rewriting the head tracker script.
- A keepalive ping-timeout (1011) on the subscriber was mitigated by disabling automatic pings and catching `ConnectionClosed` to exit cleanly.

Next recommended actions:

- Replace the placeholder center-of-face yaw/pitch math in `sensors/head_tracker.py` with a proper `solvePnP`-based pose estimation using canonical face landmarks (high priority).
- Optionally tune the sender `websockets.connect` ping intervals on `head_tracker.py` if ping-timeout issues reappear under sustained network load.
- Add a small operator visual (ASCII or minimal web debug page) to confirm recenter and yaw/pitch values during testing.

For run steps and short sensor instructions, see `STATUS_REPORT.md` and the README updates in this commit.

**9\. Deployment and Lifecycle**

### **9.1 Static Compilation**

To ease deployment, the Rust backend should be compiled using the musl target (x86\_64-unknown-linux-musl). This creates a statically linked binary that does not depend on system libraries (like glibc), making it portable across different Linux distributions (e.g., if the host is a Raspberry Pi or a specific Linux distro). Note that rocksdb and whisper C++ libs will be bundled into this binary.

### **9.2 Directory Structure**

The system functions as a self-contained unit:

/aura-1/  
├── aura\_backend\_binary \# The Rust Executable  
├── assets/  
│ ├── index.html \# Three.js Frontend  
│ ├── dist/ \# JS/CSS bundles  
│ └── models/ \# 3D Cockpit Assets  
├── models/  
│ ├── ggml-base.bin \# Whisper Model  
│ └── llama3.gguf \# Ollama Model (or pointer to Ollama dir)  
├── data/  
│ ├── rocksdb/ \# The DB storage  
│ └── tantivy/ \# The Search Index  
└── certs/  
├── key.pem  
└── cert.pem

## **10\. Conclusion**

The transition of the AURA-1 Cockpit to a Rust-based infrastructure addresses the critical bottlenecks of the legacy system. By embedding **RocksDB** and implementing **Merkle Mountain Ranges**, the system achieves a level of data integrity and auditability suitable for high-assurance applications. The integration of **Tantivy** brings powerful semantic search capabilities entirely offline. While the frontend shift to **Three.js** is necessitated by platform constraints on iOS, it results in a more robust and compliant web application.

The recommended architecture utilizes **Tokio** and **Axum** to master the concurrency challenges of real-time audio and telemetry, ensuring that the heavy lifting of AI inference never compromises the fluidity of the user experience. This report provides the blueprint for building a secure, intelligent, and responsive system that operates fully independently of the cloud.

## ---

**Appendix A: Rust Implementation Reference**

### **A.1 Cargo Dependencies**

Ini, TOML

\[package\]  
name \= "aura-backend"  
version \= "0.1.0"  
edition \= "2021"

\[dependencies\]  
tokio \= { version \= "1.0", features \= \["full"\] }  
axum \= { version \= "0.7", features \= \["ws"\] }  
axum-server \= { version \= "0.6", features \= \["tls-rustls"\] }  
rocksdb \= "0.21"  
rs-merkle \= "1.4"  
tantivy \= { version \= "0.22", features \= \["mmap"\] }  
ollama-rs \= "0.1"  
whisper-rs \= "0.11"  
serde \= { version \= "1.0", features \= \["derive"\] }  
serde\_json \= "1.0"  
bincode \= "1.3"

### **A.2 RocksDB-MMR Integration Snippet**

Rust

// Simplified illustration of appending to the Merkle-backed Log  
pub fn append\_log(  
    db: \&DB,   
    cf\_logs: \&ColumnFamily,   
    cf\_merkle: \&ColumnFamily,   
    content: &str  
) \-\> Result\<u64, Error\> {  
    // 1\. Write Log  
    let log\_id \= get\_next\_id(db)?;  
    db.put\_cf(cf\_logs, log\_id.to\_be\_bytes(), content.as\_bytes())?;

    // 2\. Update Merkle Tree  
    let hash \= blake3::hash(content.as\_bytes());  
    let mut mmr \= load\_mmr(db, cf\_merkle)?;  
    mmr.push(hash)?;  
      
    // 3\. Commit new root  
    let new\_root \= mmr.get\_root()?;  
    save\_state(db, "merkle\_root", new\_root)?;  
      
    Ok(log\_id)  
}

#### **Works cited**

1. Low latency network service libraries? : r/rust \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/rust/comments/gnap4k/low\_latency\_network\_service\_libraries/](https://www.reddit.com/r/rust/comments/gnap4k/low_latency_network_service_libraries/)  
2. Create a WebSocket Server in Axum \- SSOJet, accessed December 29, 2025, [https://ssojet.com/websocket/create-a-websocket-server-in-axum/](https://ssojet.com/websocket/create-a-websocket-server-in-axum/)  
3. Rust RocksDb — MultiGet & MergeOperator | by Hiraq Citra M | lifefunk \- Medium, accessed December 29, 2025, [https://medium.com/lifefunk/rust-rocksdb-multiget-mergeoperator-1ef6ea7e0748](https://medium.com/lifefunk/rust-rocksdb-multiget-mergeoperator-1ef6ea7e0748)  
4. Tantivy is a full-text search engine library inspired by Apache Lucene and written in Rust \- GitHub, accessed December 29, 2025, [https://github.com/quickwit-oss/tantivy](https://github.com/quickwit-oss/tantivy)  
5. merklemountainrange \- Rust \- Docs.rs, accessed December 29, 2025, [https://docs.rs/merklemountainrange/latest/merklemountainrange/](https://docs.rs/merklemountainrange/latest/merklemountainrange/)  
6. bilinearlabs/rs-merkle-tree: Merkle tree implementation in Rust with configurable storage backends and hash functions. Fixed depth and incremental only. Optimized for fast proof generation. \- GitHub, accessed December 29, 2025, [https://github.com/bilinearlabs/rs-merkle-tree](https://github.com/bilinearlabs/rs-merkle-tree)  
7. A generalized merkle mountain range implementation. \- GitHub, accessed December 29, 2025, [https://github.com/nervosnetwork/merkle-mountain-range](https://github.com/nervosnetwork/merkle-mountain-range)  
8. thyeem/monotree: An optimized Sparse Merkle Tree in Rust \- GitHub, accessed December 29, 2025, [https://github.com/thyeem/monotree](https://github.com/thyeem/monotree)  
9. Tantivy 0.7 released : r/rust \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/rust/comments/9g89fu/tantivy\_07\_released/](https://www.reddit.com/r/rust/comments/9g89fu/tantivy_07_released/)  
10. Tantivy 0.22 | Quickwit, accessed December 29, 2025, [https://quickwit.io/blog/tantivy-0.22](https://quickwit.io/blog/tantivy-0.22)  
11. langchain-rust/examples/embedding\_ollama.rs at main \- GitHub, accessed December 29, 2025, [https://github.com/Abraxas-365/langchain-rust/blob/main/examples/embedding\_ollama.rs](https://github.com/Abraxas-365/langchain-rust/blob/main/examples/embedding_ollama.rs)  
12. LanceDB vs Vearch on Vector Search Capabilities \- Zilliz blog, accessed December 29, 2025, [https://zilliz.com/blog/lance-db-vs-vearch-a-comprehensive-vector-database-comparison](https://zilliz.com/blog/lance-db-vs-vearch-a-comprehensive-vector-database-comparison)  
13. Build a Fast and Lightweight Rust Vector Search App with Rig & LanceDB \- DEV Community, accessed December 29, 2025, [https://dev.to/0thtachi/build-a-fast-and-lightweight-rust-vector-search-app-with-rig-lancedb-57h2](https://dev.to/0thtachi/build-a-fast-and-lightweight-rust-vector-search-app-with-rig-lancedb-57h2)  
14. api\_ollama \- Rust \- Docs.rs, accessed December 29, 2025, [https://docs.rs/api\_ollama](https://docs.rs/api_ollama)  
15. pepperoni21/ollama-rs: A simple and easy-to-use library for interacting with the Ollama API., accessed December 29, 2025, [https://github.com/pepperoni21/ollama-rs](https://github.com/pepperoni21/ollama-rs)  
16. how to maintain chat sessions. : r/ollama \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/ollama/comments/1g3nxup/how\_to\_maintain\_chat\_sessions/](https://www.reddit.com/r/ollama/comments/1g3nxup/how_to_maintain_chat_sessions/)  
17. whisper-rs/examples/basic\_use.rs at master \- GitHub, accessed December 29, 2025, [https://github.com/tazz4843/whisper-rs/blob/master/examples/basic\_use.rs](https://github.com/tazz4843/whisper-rs/blob/master/examples/basic_use.rs)  
18. rhasspy/piper: A fast, local neural text to speech system \- GitHub, accessed December 29, 2025, [https://github.com/rhasspy/piper](https://github.com/rhasspy/piper)  
19. Rust Web Frameworks Compared: Actix vs Axum vs Rocket \- DEV Community, accessed December 29, 2025, [https://dev.to/leapcell/rust-web-frameworks-compared-actix-vs-axum-vs-rocket-4bad](https://dev.to/leapcell/rust-web-frameworks-compared-actix-vs-axum-vs-rocket-4bad)  
20. Axum vs Actix : r/rust \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/rust/comments/1b216bf/axum\_vs\_actix/](https://www.reddit.com/r/rust/comments/1b216bf/axum_vs_actix/)  
21. I really really really want to use Godot, but sadly, the support for web exports... | Hacker News, accessed December 29, 2025, [https://news.ycombinator.com/item?id=39402953](https://news.ycombinator.com/item?id=39402953)  
22. Exporting for the Web in Godot 4.3 \- do HTML5 exports work for macOS and iOS \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/godot/comments/1etwszt/exporting\_for\_the\_web\_in\_godot\_43\_do\_html5/](https://www.reddit.com/r/godot/comments/1etwszt/exporting_for_the_web_in_godot_43_do_html5/)  
23. Godot 4.x web export problem on MacOS \- Reddit, accessed December 29, 2025, [https://www.reddit.com/r/godot/comments/1dn4ybc/godot\_4x\_web\_export\_problem\_on\_macos/](https://www.reddit.com/r/godot/comments/1dn4ybc/godot_4x_web_export_problem_on_macos/)  
24. WebGL Content Error on iOS Devices \- Need Help\! \- Godot Forum, accessed December 29, 2025, [https://forum.godotengine.org/t/webgl-content-error-on-ios-devices-need-help/80213](https://forum.godotengine.org/t/webgl-content-error-on-ios-devices-need-help/80213)  
25. Barrel Distortion, accessed December 29, 2025, [https://prideout.net/barrel-distortion](https://prideout.net/barrel-distortion)  
26. Self-Signed Certificates — Kurento 7.3 documentation \- Read the Docs, accessed December 29, 2025, [https://doc-kurento.readthedocs.io/en/stable/knowledge/selfsigned\_certs.html](https://doc-kurento.readthedocs.io/en/stable/knowledge/selfsigned_certs.html)  
27. FiloSottile/mkcert: A simple zero-config tool to make locally trusted development certificates with any names you'd like. \- GitHub, accessed December 29, 2025, [https://github.com/FiloSottile/mkcert](https://github.com/FiloSottile/mkcert)  
28. Self signed SSL certificates on iOS \- Jozef Cipa, accessed December 29, 2025, [https://jozefcipa.com/blog/self-signed-ssl-certificates-on-ios/](https://jozefcipa.com/blog/self-signed-ssl-certificates-on-ios/)
