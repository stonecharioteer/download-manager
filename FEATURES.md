# Download Manager

This is a learning exercise for Rust.

### Phase 1

**Task 1 — Basic blocking single-file download (MVP)** (1) Build: A CLI tool
`dlm` that downloads the Alpine ISO using `reqwest::blocking` and saves it to
disk. Show progress via simple byte counter printed every second. (2)
Google/read:

- “reqwest blocking download file example”
- “Rust clap derive subcommand example”
- “std::fs::File write_all vs write” (3) Self-check:
- Does it resume cleanly on network failure?
- Does progress update correctly in bytes/sec?
- Is output file identical in size to `Content-Length`?

---

**Task 2 — Switch to async + tokio** (1) Build: Convert the blocking downloader
into an async version using `reqwest` + `tokio::fs`. Keep same progress
reporting logic but via `tokio::time::interval`. (2) Google/read:

- “reqwest async stream response body”
- “tokio::io::AsyncWriteExt example”
- “tokio::time::interval usage” (3) Self-check:
- Are you awaiting everything correctly (no blocking code)?
- Does it use <100% CPU while downloading?
- Can you interrupt with Ctrl+C safely (graceful shutdown)?

---

**Task 3 — Ranged + chunked download skeleton** (1) Build: Modify the downloader
to accept a `--range-start` and `--range-end` flag. Use `Range` header in
request. Just download that part of the file to a temp file. (2) Google/read:

- “reqwest set header Range bytes example”
- “HTTP Range header format”
- “fs::OpenOptions append/truncate mode” (3) Self-check:
- Does server return `206 Partial Content`?
- Is downloaded chunk size correct?
- Can multiple non-overlapping ranges be stitched together with `cat`?

**Task 4 — Concurrent chunk workers (worker pool, no orchestration yet)** (1)
Build: Split the file into N equal ranges (configurable `--workers N`). Spawn N
`tokio::spawn` tasks, each downloading its assigned range with `Range` header
into distinct temp files (`part.0`, `part.1`, ...). Wait for all tasks and then
_do not_ concatenate yet — just verify parts. Use the Alpine ISO as the test
file. (2) Google/read:

- “reqwest Range header partial content example”
- “tokio::spawn examples”
- “futures::future::join_all usage” (3) Self-check:
- Do all parts complete and produce the expected byte lengths?
- Does total time improve vs single-worker run?
- Are there no clashes writing to the same file (each worker uses its own part
  file)?

---

**Task 5 — Shared progress aggregation with Arc/Mutex (live telemetry)** (1)
Build: Add a shared progress structure `Arc<tokio::sync::Mutex<Progress>>`
updated by each worker as it writes bytes. Run a single `tokio::task` that reads
this shared state every 200–500ms and prints aggregated percent, bytes/sec, ETA.
Use atomic counters if you prefer. (2) Google/read:

- “Arc tokio::sync::Mutex pattern”
- “std::sync::atomic AtomicU64 example”
- “how to compute moving bytes/sec for progress bar” (3) Self-check:
- Does aggregated progress equal sum of per-part bytes written?
- Does the progress reporter never block workers (no long-held locks)?
- Does ETA/bytes-per-sec look reasonable and update smoothly?

---

**Task 6 — Pause + resume (persist simple state file)** (1) Build: Implement a
pause handler: on SIGINT (or a `pause` CLI command) write a JSON state file
listing total size, worker ranges, and bytes completed per part; stop workers
cleanly. Implement resume: read state file and only download remaining bytes for
incomplete parts, appending to existing part files. Verify final concatenation
yields a full ISO. (2) Google/read:

- “serde_json write/read file example”
- “signal handling tokio ctrlc or signal-hook”
- “atomic rename write temp file then rename (durable state write)” (3)
  Self-check:
- After pausing, is the state file a truthful snapshot of progress?
- After resuming, are only remaining bytes fetched and parts appended correctly?
- After concatenation, is the final ISO the correct size and checksum (matches
  Content-Length / checksums)?

**Task 7 — Merge and verify parts (assembly + checksum)** (1) Build: After all
chunk files finish, merge them in order into a single `.iso` file. Compute and
print the SHA256 checksum for the merged file. Optionally verify it against a
known checksum (if available). (2) Google/read:

- “Rust concatenate multiple files to one”
- “std::fs::File seek and write_all example”
- “sha2 crate example for file hashing” (3) Self-check:
- Is the merged file size equal to total Content-Length?
- Does the SHA256 checksum match the expected one?
- Are temporary part files safely removed afterward?

---

**Task 8 — Proper error handling with `thiserror` + `anyhow`** (1) Build: Define
a custom `DownloadError` enum with variants like `Network`, `IO`,
`InvalidRange`, etc. Convert your current `.expect()` and `unwrap()` usages into
proper error propagation with `anyhow::Result`. (2) Google/read:

- “thiserror custom error example”
- “anyhow context usage”
- “? operator vs anyhow::Result” (3) Self-check:
- Can each major failure print a clean, contextual error message?
- Are worker errors surfaced and logged without panic?
- Can you still resume after an error without data corruption?

---

**Task 9 — Add structured telemetry via `tracing`** (1) Build: Replace printlns
with `tracing` spans and events. Add a global subscriber
(`tracing_subscriber::fmt`) and structured fields for worker id, bytes_written,
and elapsed_time. (2) Google/read:

- “tracing spans and events example”
- “tracing_subscriber::fmt::init usage”
- “tracing structured logging with fields” (3) Self-check:
- Are logs tagged with worker IDs and timestamps?
- Does enabling `RUST_LOG=debug` show detailed tracing output?
- Does performance remain unaffected when tracing is enabled?

**Task 10 — Refactor shared state with `Arc<Mutex>` for clean coordination** (1)
Build: Move progress, worker metadata, and control flags (e.g. paused, canceled)
into a central `SharedState` struct wrapped in `Arc<tokio::sync::Mutex<_>>`.
Pass clones to each worker. Centralize updates and access patterns. (2)
Google/read:

- “tokio::sync::Mutex lock scope best practices”
- “derive(Clone) for structs with Arc fields”
- “how to avoid deadlocks with async Mutex” (3) Self-check:
- Does every worker modify shared state safely without blocking others?
- Are locks held for minimal time (no async work while locked)?
- Can you easily add new shared fields (e.g. pause flag) without breaking flow?

---

**Task 11 — Axum HTTP control plane (pause/resume API)** (1) Build: Add an Axum
server with two endpoints: `POST /pause` and `POST /resume`. These should modify
the shared state (pause flag) and trigger appropriate behavior in workers. Run
it alongside the downloader in the same process with `tokio::select!`. (2)
Google/read:

- “axum shared state with Arc<Mutex> example”
- “tokio::select! multiple async tasks”
- “axum router post handler json response example” (3) Self-check:
- Can you pause/resume downloads via curl without killing the app?
- Does the HTTP server stay responsive during downloads?
- Are workers reacting promptly (within 1–2 seconds) to pause/resume?

---

**Task 12 — gRPC streaming progress via Tonic** (1) Build: Add a gRPC service
using `tonic` that streams progress updates. Define a simple proto with
`Progress { bytes_downloaded, total_bytes, percent }`. Broadcast from progress
reporter to all connected gRPC clients. (2) Google/read:

- “tonic bidirectional or server streaming example”
- “broadcast channel for live updates tokio::sync::broadcast”
- “generate tonic code build.rs example” (3) Self-check:
- Does a connected gRPC client receive continuous progress updates?
- Can multiple clients subscribe concurrently without issues?
- Is the stream properly closed when the download finishes or is canceled?
