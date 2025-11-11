# Download Manager - Task List

## Phase 1: Core Functionality

### Task 1 — Basic blocking single-file download (MVP)

**Build Requirements:**
- [x] CLI tool `dlm` that downloads using `reqwest::blocking`
- [x] Saves file to disk
- [x] Show progress via simple byte counter printed every second

**Google/Read Topics:**
- [x] "reqwest blocking download file example"
- [x] "Rust clap derive subcommand example"
- [x] "std::fs::File write_all vs write"

**Self-Check:**
- [x] Does it resume cleanly on network failure?
- [x] Does progress update correctly in bytes/sec?
- [x] Is output file identical in size to `Content-Length`?

---

### Task 2 — Switch to async + tokio

**Build Requirements:**
- [x] Convert the blocking downloader into an async version
- [x] Use `reqwest` (async) + `tokio::fs`
- [x] Keep same progress reporting logic but via `tokio::time::interval`

**Google/Read Topics:**
- [x] "reqwest async stream response body"
- [x] "tokio::io::AsyncWriteExt example"
- [x] "tokio::time::interval usage"

**Self-Check:**
- [x] Are you awaiting everything correctly (no blocking code)?
- [x] Does it use <100% CPU while downloading?
- [x] Can you interrupt with Ctrl+C safely (graceful shutdown)?

---

### Task 3 — Ranged + chunked download skeleton

**Build Requirements:**
- [ ] Accept a `--range-start` flag
- [ ] Accept a `--range-end` flag
- [x] Use `Range` header in request
- [ ] Download only that part of the file to a temp file

**Google/Read Topics:**
- [x] "reqwest set header Range bytes example"
- [x] "HTTP Range header format"
- [x] "fs::OpenOptions append/truncate mode"

**Self-Check:**
- [x] Does server return `206 Partial Content`?
- [ ] Is downloaded chunk size correct?
- [ ] Can multiple non-overlapping ranges be stitched together with `cat`?

---

### Task 4 — Concurrent chunk workers

**Build Requirements:**
- [ ] Add `--workers N` flag
- [ ] Split file into N equal ranges
- [ ] Spawn N `tokio::spawn` tasks for concurrent downloads
- [ ] Each worker downloads to distinct temp file (`part.0`, `part.1`, etc.)
- [ ] Use `futures::future::join_all` to wait for all workers
- [ ] Each worker computes SHA256 for its chunk (for verification)

**Google/Read Topics:**
- [x] "reqwest Range header partial content example"
- [ ] "tokio::spawn examples"
- [ ] "futures::future::join_all usage"

**Self-Check:**
- [ ] Do all parts complete with expected byte lengths?
- [ ] Does total time improve vs single-worker?
- [ ] No file write clashes (each worker uses own part file)?

---

### Task 5 — Shared progress aggregation

**Build Requirements:**
- [ ] Add `Arc<tokio::sync::Mutex<Progress>>` for shared progress
- [ ] Workers update shared state as they write bytes
- [ ] Progress task reads state every 200-500ms
- [ ] Display aggregated percent, bytes/sec, ETA

**Google/Read Topics:**
- [ ] "Arc tokio::sync::Mutex pattern"
- [ ] "std::sync::atomic AtomicU64 example"
- [ ] "how to compute moving bytes/sec for progress bar"

**Self-Check:**
- [ ] Aggregated progress equals sum of per-part bytes?
- [ ] Progress reporter never blocks workers?
- [ ] ETA/bytes-per-sec updates smoothly?

---

### Task 6 — Pause + resume with state persistence

**Build Requirements:**
- [ ] Handle SIGINT to write JSON state file
- [ ] State file lists: total size, worker ranges, bytes per part, per-chunk SHA256
- [ ] Resume: read state, download remaining bytes, append to parts
- [ ] Resume: continue SHA256 calculation from pause point
- [ ] Verify final concatenation yields correct file

**Google/Read Topics:**
- [ ] "serde_json write/read file example"
- [x] "signal handling tokio ctrlc or signal-hook"
- [ ] "atomic rename write temp file"

**Self-Check:**
- [ ] State file is truthful snapshot of progress?
- [ ] Resume fetches only remaining bytes?
- [ ] SHA256 calculation resumes correctly for partial chunks?
- [ ] Final file has correct size and checksum?

---

### Task 7 — Merge and verify parts

**Build Requirements:**
- [ ] Merge chunk files in order into single file
- [ ] Compute overall SHA256: combine per-chunk hashes OR hash merged file
- [ ] Optionally verify against known checksum
- [ ] Remove temporary part files after merge

**Google/Read Topics:**
- [ ] "Rust concatenate multiple files to one"
- [ ] "std::fs::File seek and write_all example"
- [x] "sha2 crate example for file hashing"
- [ ] "SHA256 combining partial hashes" (note: may need to hash merged file instead)

**Self-Check:**
- [ ] Merged file size equals total Content-Length?
- [ ] SHA256 checksum matches expected (verified against known hash)?
- [ ] Temp part files safely removed?

---

### Task 8 — Code modularization and organization

**Build Requirements:**
- [x] Create module structure: `cli.rs`, `download/`, `download/mod.rs`
- [x] Move download functions to `download` module
- [x] Extract CLI types to `cli` module
- [x] Keep `main.rs` minimal (just setup and dispatch)
- [x] Add `Cli::execute()` method for clean main
- [x] Use `Url` type for type-safe URL validation
- [x] Extract common utils to `download/utils.rs`:
  - [x] Filename extraction from URL (`extract_filename_from_url`)
  - [x] SHA256 hashing post-download (`hashfile`)
  - [x] Simplified hashing approach (compute after download, not streaming)
  - [ ] File opening logic (resume/overwrite/create) - deferred
  - [ ] Progress message formatting - deferred
- [x] Decouple progress/interrupt from download functions:
  - [x] Use `Arc<AtomicUsize>` for progress tracking (`DownloadProgress` struct)
  - [x] Spawn separate task in `execute()` for progress updates
  - [x] Download functions only update atomics, no bar logic
  - [x] Interrupt checking via atomic flag only
  - [x] Progress reporter shows speed and bytes downloaded
- [x] Download functions return `PathBuf` instead of `()`
- [x] CLI handles hashing and printing, not download functions
- [x] Streamline stdout: consistent messages across blocking/async
- [x] Show total download time (excluding hash time)

**Google/Read Topics:**
- [x] "Rust module system mod.rs vs file.rs"
- [x] "Rust pub use re-exports"
- [x] "structuring larger Rust projects"
- [x] "Arc AtomicUsize for progress tracking"

**Self-Check:**
- [x] Each module has single clear responsibility?
- [x] No circular dependencies between modules?
- [x] Public API surface is minimal and clear?
- [x] Download functions have no UI concerns?

---

### Task 9 — Error handling with thiserror + anyhow

**Build Requirements:**
- [ ] Define `DownloadError` enum with variants (Network, IO, InvalidRange, etc.)
- [ ] Convert `.expect()` and `unwrap()` to proper error propagation
- [ ] Worker errors surfaced without panic

**Google/Read Topics:**
- [ ] "thiserror custom error example"
- [ ] "anyhow context usage"
- [ ] "? operator vs anyhow::Result"

**Self-Check:**
- [x] Clean, contextual error messages for failures?
- [ ] Worker errors logged without panic?
- [ ] Can resume after error without corruption?

---

### Task 10 — Structured telemetry via tracing

**Build Requirements:**
- [ ] Replace printlns with `tracing` spans and events
- [ ] Add global subscriber (`tracing_subscriber::fmt`)
- [ ] Structured fields: worker_id, bytes_written, elapsed_time

**Google/Read Topics:**
- [ ] "tracing spans and events example"
- [ ] "tracing_subscriber::fmt::init usage"
- [ ] "tracing structured logging with fields"

**Self-Check:**
- [ ] Logs tagged with worker IDs and timestamps?
- [ ] `RUST_LOG=debug` shows detailed output?
- [ ] Performance unaffected when tracing enabled?

---

### Task 11 — Refactor shared state coordination

**Build Requirements:**
- [ ] Move progress, metadata, control flags to `SharedState` struct
- [ ] Wrap in `Arc<tokio::sync::Mutex<_>>`
- [ ] Pass clones to each worker
- [ ] Centralize updates and access patterns
- [ ] Consider extracting to `state` module

**Google/Read Topics:**
- [ ] "tokio::sync::Mutex lock scope best practices"
- [ ] "derive(Clone) for structs with Arc fields"
- [ ] "how to avoid deadlocks with async Mutex"

**Self-Check:**
- [ ] Workers modify shared state safely?
- [ ] Locks held for minimal time?
- [ ] Easy to add new shared fields?

---

### Task 12 — Axum HTTP control plane

**Build Requirements:**
- [ ] Add Axum server with `POST /pause` and `POST /resume`
- [ ] Endpoints modify shared state (pause flag)
- [ ] Run alongside downloader with `tokio::select!`
- [ ] Extract HTTP server logic to `server` or `api` module

**Google/Read Topics:**
- [ ] "axum shared state with Arc<Mutex> example"
- [x] "tokio::select! multiple async tasks"
- [ ] "axum router post handler json response example"

**Self-Check:**
- [ ] Pause/resume via curl works?
- [ ] HTTP server stays responsive during downloads?
- [ ] Workers react within 1-2 seconds?

---

### Task 13 — gRPC streaming progress via Tonic

**Build Requirements:**
- [ ] Add gRPC service using `tonic` for progress streaming
- [ ] Define proto: `Progress { bytes_downloaded, total_bytes, percent }`
- [ ] Broadcast from progress reporter to gRPC clients
- [ ] Extract gRPC service logic to `grpc` module
- [ ] Proto definitions in `proto/` directory

**Google/Read Topics:**
- [ ] "tonic bidirectional or server streaming example"
- [ ] "broadcast channel for live updates tokio::sync::broadcast"
- [ ] "generate tonic code build.rs example"

**Self-Check:**
- [ ] Connected gRPC client receives continuous updates?
- [ ] Multiple clients can subscribe concurrently?
- [ ] Stream properly closed when download finishes?

---

## Bonus Features Implemented

- [x] `--resume` flag for resuming interrupted downloads
- [x] `--overwrite` flag for overwriting existing files
- [x] HTTP Range header support for partial downloads
- [x] HTTP status code validation (206 Partial, 416 Range Not Satisfiable, 200 Full File)
- [x] SHA256 hash verification after download
- [x] Streaming SHA256 calculation (hashes existing partial file on resume)
- [x] Ctrl-C interrupt handling with graceful shutdown
- [x] Progress spinner using indicatif library
- [x] Human-readable byte formatting (HumanBytes)
- [x] Download speed calculation and display
- [x] Edge case handling (detects when trying to resume already-complete file)
- [x] Error handling with `anyhow`
