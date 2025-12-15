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

- [x] Accept a `--range-start` flag
- [x] Accept a `--range-end` flag
- [x] Use `Range` header in request
- [x] Download only that part of the file to a temp file (`part.{start}-{end}`)
- [x] Fix implementation issues in `async_range.rs`
- [x] Design unified progress tracking system (extends indicatif):
  - [x] Create `ChunkProgressBar` wrapper that extends indicatif::ProgressBar
  - [x] Track chunk states: pending, downloading, completed, failed
  - [x] Generate custom messages/formats for indicatif to render
  - [x] Adaptive progress display modes:
    - [x] **Multi-chunk mode**: Visual bar with color-coded chunks (for
          concurrent downloads)
      - Green `█` = completed
      - Yellow/Cyan/Magenta `█` = actively downloading (different colors for
        different workers)
      - Gray `░` = pending
      - Red `█` = failed
      - Example: `[████░░██░░] 45% @ 5.2 MB/s`
    - [x] **Single-chunk mode**: Regular spinner (for single-worker downloads)
  - [x] Works for blocking, async, and range downloads
  - [x] Wraps indicatif::ProgressBar, uses `set_message()` with custom
        formatting
  - [x] Use colored crate for ANSI color codes in messages

**Google/Read Topics:**

- [x] "reqwest set header Range bytes example"
- [x] "HTTP Range header format"
- [x] "fs::OpenOptions append/truncate mode"
- [x] "indicatif custom progress bar rendering"
- [x] "Rust enum for state machine (chunk states)"
- [x] "colored crate Rust terminal colors"
- [x] "ANSI escape codes terminal colors"

**Self-Check:**

- [x] Does server return `206 Partial Content`?
- [x] Is downloaded chunk size correct?
- [x] Can multiple non-overlapping ranges be stitched together?
- [x] Does chunk progress bar show visual status of all ranges?

---

### Task 4 — Concurrent chunk workers

**Build Requirements:**

- [x] Add `--workers N` flag
- [x] Split file into N equal ranges
- [x] Spawn N `tokio::spawn` tasks for concurrent downloads
- [x] Each worker downloads to distinct temp file (`filename.part.start-end`)
- [x] Use `futures::future::join_all` to wait for all workers
- [x] Merge parts in correct order after all complete
- [x] SHA256 verification of merged file
- [x] Add `--no-cleanup` flag for debugging (keeps part files)
- [x] Use GET request to get Content-Length (HEAD doesn't always return it)

**Google/Read Topics:**

- [x] "reqwest Range header partial content example"
- [x] "tokio::spawn examples"
- [x] "futures::future::join_all usage"

**Self-Check:**

- [x] Do all parts complete with expected byte lengths?
- [x] Does total time improve vs single-worker?
- [x] No file write clashes (each worker uses own part file)?
- [x] Parts merged in correct order?
- [x] Final file SHA256 matches reference download?

---

### Task 5 — Shared progress aggregation

**Build Requirements:**

- [x] Add shared progress tracking (`ChunkProgressBar` with atomics)
- [x] Workers update shared state as they write bytes
- [x] Progress task reads state every 100ms
- [x] Display aggregated bytes/sec and progress visualization
- [x] Implement `ProgressTracker` trait for abstraction
- [x] Create `get_content_length()` helper for clean separation

**Google/Read Topics:**

- [x] "Arc tokio::sync::Mutex pattern"
- [x] "std::sync::atomic AtomicU64 example"
- [x] "how to compute moving bytes/sec for progress bar"

**Self-Check:**

- [x] Aggregated progress equals sum of per-part bytes?
- [x] Progress reporter never blocks workers?
- [x] Bytes-per-sec updates smoothly?

---

### Task 6 — Pause + resume with state persistence

**Build Requirements:**

- [ ] Create `DownloadState` struct with `serde` derives for state persistence
  - Fields: url, total_bytes, num_workers, chunk_ranges (with ChunkRange struct)
  - ChunkRange: id, start, end, status (Pending/Completed/Failed),
    bytes_downloaded
- [ ] Implement progress file writing on worker completion
  - Update state file atomically each time a chunk completes
  - Use atomic file write pattern (write to temp → rename)
- [ ] Add progress file naming based on URL hash
  - Format: `.dlm-progress-<url-hash>.json` in target directory
  - Multiple downloads can coexist with different state files
- [ ] Implement two-stage Ctrl-C handling
  - First Ctrl-C: Set pause flag, workers exit gracefully, keep state file
  - Second Ctrl-C: Clean exit (state already saved)
  - Print "Download paused. Run with --resume to continue."
- [ ] Implement resume logic
  - Check for existing progress file on startup with `--resume`
  - Read state from JSON
  - Calculate which chunks are incomplete
  - Spawn workers only for incomplete chunks
  - Continue updating progress file as new chunks complete
- [ ] Test pause and resume with multi-worker downloads
  - Pause mid-download, verify state file accuracy
  - Resume and verify only incomplete chunks are downloaded
  - Final file SHA256 matches expected hash

**Google/Read Topics:**

- [ ] "serde_json write/read file example"
- [x] "signal handling tokio ctrlc or signal-hook"
- [ ] "atomic rename write temp file (durable writes)"
- [ ] "SHA256 hash URL for unique filename"

**Self-Check:**

- [ ] State file is truthful snapshot of progress?
- [ ] Resume fetches only remaining bytes for incomplete chunks?
- [ ] Progress file updated atomically without corruption?
- [ ] Final file has correct size and checksum after pause/resume?
- [ ] Can pause and resume multiple times?

---

### Task 7 — Merge and verify parts

**Build Requirements:**

- [x] Merge chunk files in order into single file
- [x] Compute overall SHA256 after merge (hashing merged file)
- [x] Remove temporary part files after merge (with `--no-cleanup` flag for
      debugging)

**Google/Read Topics:**

- [x] "Rust concatenate multiple files to one"
- [x] "std::fs::File seek and write_all example"
- [x] "sha2 crate example for file hashing"

**Self-Check:**

- [x] Merged file size equals total Content-Length?
- [x] SHA256 checksum computed correctly?
- [x] Temp part files safely removed (or kept with --no-cleanup)?

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

- [ ] Define `DownloadError` enum with variants (Network, IO, InvalidRange,
      etc.)
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
- [x] HTTP status code validation (206 Partial, 416 Range Not Satisfiable, 200
      Full File)
- [x] SHA256 hash verification after download
- [x] Streaming SHA256 calculation (hashes existing partial file on resume)
- [x] Ctrl-C interrupt handling with graceful shutdown
- [x] Progress spinner using indicatif library
- [x] Human-readable byte formatting (HumanBytes)
- [x] Download speed calculation and display
- [x] Edge case handling (detects when trying to resume already-complete file)
- [x] Error handling with `anyhow`
