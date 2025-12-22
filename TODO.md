# Download Manager - Task List

**Timeline: 15-20 days (December 18, 2024 - January 6, 2025)**

**Goal: Build a multi-protocol download manager daemon with production-grade architecture, documented through "The Download Book"**

**Approach: Book-Driven Development** - Each chapter represents a complete development milestone. Complete the implementation, then write the chapter documenting what you built and learned.

---

## Book Structure & Task Mapping

### Chapter 1: What _is_ a download, anyway?
**Status:** ✅ 80% Complete (existing work)
**Tasks:** 1-8 (HTTP foundation)
**Timeline:** Days 1-2 (polish + pause/resume)

### Chapter 2: Gone, gone the form of man, rise the Daemon
**Status:** 📝 Not Started
**Tasks:** 9, 10, 11, 16, 17, 22 (daemon architecture)
**Timeline:** Days 3-7 (5 days - this is the heavy lift)

### Chapter 3: FTP or Bust
**Status:** 📝 Not Started
**Tasks:** 12 (FTP/FTPS implementation)
**Timeline:** Days 8-9 (2 days)

### Chapter 4: BitTorrent: 2000 Lines of Controlled Chaos
**Status:** 📝 Not Started  
**Tasks:** 19 (BitTorrent + tracker protocol)
**Timeline:** Days 10-15 (6 days - most complex protocol)

### Chapter 5: IPFS: Files Have Fingerprints Now, Apparently
**Status:** 📝 Not Started
**Tasks:** 20 (IPFS implementation)
**Timeline:** Days 16-17 (2 days)

### Chapter 6: S3 and Friends: The Cloud Wants Your Files
**Status:** 📝 Not Started
**Tasks:** 15 (S3 implementation)
**Timeline:** Day 18 (1 day - leverage existing HTTP knowledge)

### Chapter 7: What I Learned (And What I'd Do Differently)
**Status:** 📝 Not Started
**Tasks:** Retrospective writing
**Timeline:** Days 19-20 (final polish, demos, launch prep)

---

## Phase 1: HTTP Foundation (Chapter 1) - Days 1-2

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

### Task 9 — Error handling with thiserror + anyhow (Day 3)

**Priority:** HIGH - Foundation for production quality

**Build Requirements:**

- [ ] Define `DownloadError` enum with variants:
  - `Network(String)` - reqwest errors, connection failures
  - `IO(std::io::Error)` - file system errors
  - `InvalidRange(String)` - range request errors
  - `UnsupportedProtocol(String)` - protocol not implemented
  - `ParseError(String)` - URL/metadata parsing
  - `AuthenticationFailed(String)` - credential issues
  - `StateCorruption(String)` - progress file issues
- [ ] Implement `thiserror::Error` derive with context
- [ ] Convert `.expect()` and `unwrap()` to proper error propagation
- [ ] Worker errors surfaced without panic (use `Result` returns)
- [ ] Add error context with `.context()` throughout codebase
- [ ] Create protocol-specific error variants for future protocols

**Google/Read Topics:**

- [ ] "thiserror custom error example"
- [ ] "anyhow context usage"
- [ ] "? operator vs anyhow::Result"
- [ ] "error handling in async Rust"
- [ ] "propagating errors from tokio::spawn"

**Self-Check:**

- [ ] Clean, contextual error messages for all failures?
- [ ] Worker errors logged without panic?
- [ ] Can resume after error without corruption?
- [ ] Error types support `Send + Sync` for async contexts?

**Book Content:** Chapter 2 section on robust error handling

---

### Task 10 — Structured telemetry via tracing (Day 3)

**Priority:** HIGH - Essential for debugging multi-protocol system

**Build Requirements:**

- [ ] Replace printlns with `tracing` spans and events
- [ ] Add global subscriber (`tracing_subscriber::fmt`)
- [ ] Structured fields: `worker_id`, `protocol`, `bytes_written`, `elapsed_time`, `url`
- [ ] Create spans for:
  - Download lifecycle (start → complete/error)
  - Worker execution (chunk download spans)
  - Protocol operations (connection, auth, transfer)
  - State persistence operations
- [ ] Add log levels appropriately (DEBUG/INFO/WARN/ERROR)
- [ ] Optional JSON output format for production logging
- [ ] Trace HTTP requests/responses at DEBUG level
- [ ] Performance metrics at INFO level

**Google/Read Topics:**

- [ ] "tracing spans and events example"
- [ ] "tracing_subscriber::fmt::init usage"
- [ ] "tracing structured logging with fields"
- [ ] "tracing async functions"
- [ ] "tracing performance overhead"

**Self-Check:**

- [ ] Logs tagged with worker IDs and timestamps?
- [ ] `RUST_LOG=debug` shows detailed output?
- [ ] `RUST_LOG=info` shows only key events?
- [ ] Performance unaffected when tracing enabled at INFO?
- [ ] Can trace individual downloads across protocol boundaries?

**Book Content:** Chapter 7 section on observability

---

## Phase 2: Protocol Abstraction & Multi-Protocol Support (Days 4-11)

### Task 11 — Protocol abstraction layer (Day 4)

**Priority:** CRITICAL - Foundation for multi-protocol architecture

**Build Requirements:**

- [ ] Design `Protocol` trait with async methods:
  ```rust
  #[async_trait]
  pub trait Protocol: Send + Sync {
      async fn metadata(&self, url: &Url) -> Result<Metadata>;
      async fn supports_resume(&self) -> bool;
      async fn download_range(&self, url: &Url, range: Range<u64>, dest: PathBuf) -> Result<u64>;
      async fn download_full(&self, url: &Url, dest: PathBuf) -> Result<u64>;
      fn protocol_name(&self) -> &'static str;
  }
  ```
- [ ] Define `Metadata` struct: `{ size: Option<u64>, supports_range: bool, content_type: Option<String> }`
- [ ] Create `ProtocolRegistry` for dynamic dispatch based on URL scheme
- [ ] Refactor existing HTTP code into `HttpProtocol` implementation
- [ ] Extract to `src/protocols/` module hierarchy:
  - `src/protocols/mod.rs` - trait definition, registry
  - `src/protocols/http.rs` - HTTP/HTTPS implementation
  - `src/protocols/ftp.rs` - FTP (to be implemented)
  - etc.
- [ ] Update CLI to use protocol abstraction
- [ ] Add `--protocol` flag override for testing

**Google/Read Topics:**

- [ ] "async_trait crate usage"
- [ ] "trait objects vs generics for plugins"
- [ ] "dynamic dispatch in Rust"
- [ ] "Rust registry pattern"

**Self-Check:**

- [ ] Can add new protocol without touching CLI code?
- [ ] HTTP still works through abstraction layer?
- [ ] Clear error when protocol unsupported?
- [ ] Protocol selection automatic based on URL scheme?

**Book Content:** Chapter 2 - "Protocol Abstraction Architecture"

---

### Task 12 — FTP/FTPS protocol implementation (Days 5-6)

**Priority:** HIGH - Demonstrates different async I/O patterns

**Build Requirements:**

- [ ] Add `suppaftp` or `async-ftp` dependency
- [ ] Implement `FtpProtocol` struct with `Protocol` trait
- [ ] Support both active and passive mode (use passive by default)
- [ ] Handle FTP authentication (username/password from URL or args)
- [ ] Support FTPS (explicit TLS) via `--ftps` flag
- [ ] Implement progress tracking during FTP transfers
- [ ] Handle FTP-specific errors (421 timeout, 550 file not found)
- [ ] Test with public FTP servers (e.g., ftp://ftp.gnu.org/)
- [ ] Support resume via REST command if server supports it
- [ ] Handle binary vs ASCII mode (always use binary for downloads)

**Google/Read Topics:**

- [ ] "suppaftp async example"
- [ ] "FTP protocol REST command"
- [ ] "FTP active vs passive mode"
- [ ] "async-native-tls with FTP"
- [ ] "FTP SIZE command for metadata"

**Self-Check:**

- [ ] Can download from public FTP servers?
- [ ] Progress tracking works during FTP transfer?
- [ ] Resume works if server supports REST?
- [ ] Clear error messages for FTP-specific failures?
- [ ] Both FTP and FTPS work?

**Book Content:** Chapter 3 - "FTP or Bust"

---

### Task 13 — SFTP protocol (over SSH) (Day 7)

**Priority:** MEDIUM - Shows SSH/async integration

**Build Requirements:**

- [ ] Add `async-ssh2-tokio` or `russh` dependency
- [ ] Implement `SftpProtocol` struct with `Protocol` trait
- [ ] SSH key-based authentication support
- [ ] Password authentication fallback
- [ ] SFTP file transfer with progress tracking
- [ ] Handle SSH connection pooling/reuse
- [ ] Support resume via file offset
- [ ] Test with local SFTP server or public test servers

**Google/Read Topics:**

- [ ] "russh async SFTP example"
- [ ] "SSH key authentication Rust"
- [ ] "SFTP protocol file transfer"
- [ ] "async-ssh2-tokio examples"

**Self-Check:**

- [ ] Can authenticate with SSH keys?
- [ ] Password auth works as fallback?
- [ ] Progress tracking functional?
- [ ] Resume works correctly?

**Book Content:** Chapter 3 - extend with SFTP section

---

### Task 14 — WebDAV protocol (Day 8)

**Priority:** MEDIUM - Reuses HTTP knowledge, shows abstraction value

**Build Requirements:**

- [ ] Implement `WebDavProtocol` using `reqwest` with WebDAV extensions
- [ ] PROPFIND for metadata (file size, content type)
- [ ] GET for download (reuse HTTP download logic)
- [ ] Handle WebDAV authentication (Basic, Digest)
- [ ] Support recursive directory listing (for future multi-file downloads)
- [ ] Test with public WebDAV servers or Nextcloud instance

**Google/Read Topics:**

- [ ] "WebDAV protocol PROPFIND"
- [ ] "reqwest WebDAV example"
- [ ] "WebDAV authentication methods"
- [ ] "Rust XML parsing for PROPFIND response"

**Self-Check:**

- [ ] Can list files via PROPFIND?
- [ ] Downloads work via GET?
- [ ] Authentication successful?
- [ ] Metadata extraction correct?

**Book Content:** Chapter 4 - "Remote Filesystems"

---

### Task 15 — S3 protocol implementation (Days 9-10)

**Priority:** HIGH - Cloud storage is critical for modern systems

**Build Requirements:**

- [ ] Add `aws-sdk-s3` or `rusoto_s3` dependency
- [ ] Implement `S3Protocol` with `Protocol` trait
- [ ] Support S3 pre-signed URLs (easiest, no auth needed)
- [ ] Support standard S3 API with access key/secret
- [ ] HeadObject for metadata
- [ ] GetObject with range support for multi-part downloads
- [ ] Handle pagination for ListObjects (future multi-file support)
- [ ] Support custom S3-compatible endpoints (MinIO, Wasabi, etc.)
- [ ] Environment variable configuration (AWS_ACCESS_KEY_ID, etc.)
- [ ] Region detection and configuration

**Google/Read Topics:**

- [ ] "aws-sdk-s3 async examples"
- [ ] "S3 presigned URL download"
- [ ] "rusoto_s3 GetObject with range"
- [ ] "S3 compatible endpoints configuration"

**Self-Check:**

- [ ] Pre-signed URLs work without credentials?
- [ ] Standard S3 API works with credentials?
- [ ] Range requests supported?
- [ ] Works with MinIO/other S3-compatible services?
- [ ] Proper error handling for missing files (404)?

**Book Content:** Chapter 5 - "Cloud Object Storage"

---

## Phase 3: Advanced Concurrency & Control Plane (Days 11-13)

### Task 16 — Refactor shared state coordination (Day 11)

**Priority:** HIGH - Required for control plane

**Build Requirements:**

- [ ] Design `DownloadManager` struct with comprehensive state:
  - Active downloads (HashMap<DownloadId, DownloadState>)
  - Worker pools per download
  - Global progress aggregation
  - Control flags (pause, cancel, rate limit)
- [ ] Wrap in `Arc<tokio::sync::RwLock<_>>` for concurrent access
- [ ] Create `DownloadHandle` for controlling individual downloads
- [ ] Implement pause/resume via state flags
- [ ] Support multiple concurrent downloads (different files)
- [ ] Add download queue with priority
- [ ] Rate limiting across all downloads (optional)
- [ ] Extract to `src/state.rs` module

**Google/Read Topics:**

- [ ] "tokio::sync::RwLock vs Mutex best practices"
- [ ] "derive(Clone) for structs with Arc fields"
- [ ] "async Mutex deadlock prevention"
- [ ] "concurrent HashMap alternatives (DashMap)"

**Self-Check:**

- [ ] Multiple downloads run concurrently?
- [ ] Pause/resume works without corruption?
- [ ] No deadlocks under load?
- [ ] Easy to query download status?

**Book Content:** Chapter 7 - "State Management"

---

### Task 17 — Axum HTTP control plane API (Day 12)

**Priority:** HIGH - Demonstrates production-ready async service

**Build Requirements:**

- [ ] Add `axum`, `tower`, `tower-http` dependencies
- [ ] Design REST API:
  - `POST /downloads` - start new download (body: {url, protocol, options})
  - `GET /downloads` - list all downloads
  - `GET /downloads/:id` - get download status
  - `POST /downloads/:id/pause` - pause download
  - `POST /downloads/:id/resume` - resume download
  - `DELETE /downloads/:id` - cancel and cleanup
  - `GET /health` - health check
  - `GET /metrics` - Prometheus-compatible metrics (optional)
- [ ] Use `Arc<DownloadManager>` shared state via Axum's State extractor
- [ ] JSON request/response bodies with `serde_json`
- [ ] WebSocket endpoint for real-time progress streaming
- [ ] CORS middleware for web UI support
- [ ] Run API server alongside CLI with `tokio::select!`
- [ ] Optional flag `--api-port` to enable HTTP API
- [ ] Extract to `src/api/` module

**Google/Read Topics:**

- [ ] "axum shared state with Arc example"
- [ ] "axum REST API design"
- [ ] "axum WebSocket streaming"
- [ ] "tower middleware composition"
- [ ] "tokio::select! for concurrent services"

**Self-Check:**

- [ ] Can start downloads via API while CLI runs?
- [ ] WebSocket streams progress in real-time?
- [ ] Pause/resume works via API?
- [ ] API stays responsive during heavy downloads?
- [ ] Proper error responses (404, 400, etc.)?

**Book Content:** Chapter 7 - "HTTP Control Plane"

---

### Task 18 — gRPC streaming progress (Day 13)

**Priority:** MEDIUM - Shows gRPC + streaming expertise

**Build Requirements:**

- [ ] Add `tonic`, `prost` dependencies
- [ ] Define protobuf schema in `proto/download.proto`:
  ```proto
  service DownloadService {
    rpc StreamProgress(ProgressRequest) returns (stream ProgressUpdate);
    rpc StartDownload(DownloadRequest) returns (DownloadResponse);
    rpc PauseDownload(DownloadId) returns (StatusResponse);
  }
  ```
- [ ] Use `tonic-build` in `build.rs`
- [ ] Implement gRPC server with `DownloadManager` state
- [ ] Use `tokio::sync::broadcast` for progress fan-out to multiple clients
- [ ] Support bidirectional streaming for interactive control
- [ ] Run gRPC server on separate port alongside HTTP API
- [ ] Optional flag `--grpc-port` to enable gRPC
- [ ] Extract to `src/grpc/` module

**Google/Read Topics:**

- [ ] "tonic server streaming example"
- [ ] "tonic-build build.rs setup"
- [ ] "tokio::sync::broadcast for fan-out"
- [ ] "tonic bidirectional streaming"

**Self-Check:**

- [ ] gRPC clients receive continuous progress?
- [ ] Multiple clients subscribe concurrently?
- [ ] Stream closes gracefully on completion?
- [ ] Works alongside HTTP API?

**Book Content:** Chapter 7 - "gRPC Streaming"

---

## Phase 4: Distributed & P2P Protocols (Days 14-17)

### Task 19 — BitTorrent protocol implementation (Days 14-15)

**Priority:** HIGH - The ultimate async/concurrency challenge

**Build Requirements:**

- [ ] Add `lava_torrent` for .torrent parsing
- [ ] Add `bencode` for protocol messages
- [ ] Implement `BitTorrentProtocol` with `Protocol` trait
- [ ] Torrent file parsing (pieces, trackers, info hash)
- [ ] Magnet link parsing (info hash, tracker URLs)
- [ ] HTTP/UDP tracker communication:
  - Announce (get peer list)
  - Scrape (get swarm stats)
- [ ] Peer wire protocol (TCP):
  - Handshake
  - Bitfield exchange
  - Request/Piece messages
  - Choke/Unchoke logic
- [ ] Piece verification (SHA1 hash per piece)
- [ ] Piece selection strategy (rarest-first or sequential for video)
- [ ] Concurrent peer connections (manage N peer workers)
- [ ] Upload to maintain ratio (seed while downloading)
- [ ] DHT support (optional, defer if time-constrained):
  - Kademlia DHT for trackerless torrents
  - `find_node`, `get_peers` queries
- [ ] Progress tracking across all pieces
- [ ] Test with legal torrents (Linux ISOs, public domain content)

**Google/Read Topics:**

- [ ] "BitTorrent protocol specification"
- [ ] "lava_torrent example usage"
- [ ] "Rust BitTorrent client architecture"
- [ ] "piece selection algorithms BitTorrent"
- [ ] "UDP tracker protocol"
- [ ] "Mainline DHT Kademlia"
- [ ] "async peer management patterns"

**Self-Check:**

- [ ] Can parse .torrent files and magnet links?
- [ ] Tracker communication successful?
- [ ] Connects to peers and exchanges bitfields?
- [ ] Downloads pieces correctly and verifies hashes?
- [ ] Progress shows overall torrent completion?
- [ ] Handles slow/disconnecting peers gracefully?
- [ ] Upload (seeding) works during download?

**Book Content:** Chapter 6 - "Swarm & Distributed Sources: BitTorrent"

---

### Task 20 — IPFS protocol support (Days 16-17)

**Priority:** MEDIUM-HIGH - Content-addressed storage is future-forward

**Build Requirements:**

- [ ] Add `ipfs-api-backend-hyper` or interface with local IPFS daemon
- [ ] Implement `IpfsProtocol` with `Protocol` trait
- [ ] Support `ipfs://` URLs (CID resolution)
- [ ] Support `ipns://` URLs (mutable names)
- [ ] Gateway fallback (public HTTP gateways like ipfs.io)
- [ ] Direct IPFS daemon API (if local daemon running)
- [ ] Libp2p integration for peer-to-peer retrieval (optional/advanced)
- [ ] CID verification after download
- [ ] Handle chunked DAG traversal for large files
- [ ] Progress tracking during multi-block retrieval

**Google/Read Topics:**

- [ ] "IPFS CID content addressing"
- [ ] "ipfs-api Rust client"
- [ ] "IPFS HTTP gateway API"
- [ ] "libp2p Rust examples"
- [ ] "IPFS DAG structure"

**Self-Check:**

- [ ] Can resolve CIDs via gateway?
- [ ] Local daemon API works if available?
- [ ] IPNS names resolve correctly?
- [ ] Large files download completely (handle DAG)?
- [ ] CID verification passes?

**Book Content:** Chapter 6 - extend with "IPFS: Content-Addressed Downloads"

---

## Phase 5: Streaming Protocols & Advanced Features (Days 18-20)

### Task 21 — HLS (m3u8) streaming protocol (Day 18)

**Priority:** MEDIUM - Common for video streaming

**Build Requirements:**

- [ ] Add `m3u8-rs` for playlist parsing
- [ ] Implement `HlsProtocol` with `Protocol` trait
- [ ] Parse master playlist (quality variants)
- [ ] Parse media playlists (segment lists)
- [ ] Download segments sequentially or concurrently
- [ ] Merge TS segments into final video file
- [ ] Handle live streams (updating playlists)
- [ ] Support AES-128 encrypted segments (key fetching)
- [ ] Progress tracking across segments

**Google/Read Topics:**

- [ ] "HLS protocol m3u8 format"
- [ ] "m3u8-rs crate example"
- [ ] "TS segment merging ffmpeg or manual"
- [ ] "HLS AES-128 decryption"

**Self-Check:**

- [ ] Can parse master and media playlists?
- [ ] Downloads all segments?
- [ ] Merges into playable video?
- [ ] Handles encrypted segments?

**Book Content:** Chapter 6 - "Streaming Protocols: HLS"

---

### Task 22 — Download queue and scheduler (Day 19)

**Priority:** HIGH - Production feature for managing many downloads

**Build Requirements:**

- [ ] Create `DownloadQueue` with priority queue (min-heap)
- [ ] Support download priorities (high, normal, low)
- [ ] Limit concurrent downloads (configurable max)
- [ ] Automatic retry with exponential backoff on failures
- [ ] Persistent queue (save to disk, restore on restart)
- [ ] Add CLI commands:
  - `dlm queue add <url> --priority=high`
  - `dlm queue list`
  - `dlm queue pause`
  - `dlm queue resume`
- [ ] Integrate with API server (queue management endpoints)

**Google/Read Topics:**

- [ ] "Rust BinaryHeap priority queue"
- [ ] "exponential backoff retry strategy"
- [ ] "persistent queue serialization"

**Self-Check:**

- [ ] Queue respects priorities?
- [ ] Concurrent limit enforced?
- [ ] Retries work with backoff?
- [ ] Queue persists across restarts?

**Book Content:** Chapter 7 - "Download Queue & Scheduling"

---

### Task 23 — Rate limiting and bandwidth control (Day 20)

**Priority:** MEDIUM - Important for production use

**Build Requirements:**

- [ ] Add `governor` or implement token bucket algorithm
- [ ] Global rate limit (across all downloads)
- [ ] Per-download rate limit
- [ ] CLI flags: `--rate-limit 5MB`, `--max-bandwidth 10MB`
- [ ] Integrate with worker byte reads (throttle before writing)
- [ ] Dynamic rate adjustment via API
- [ ] Show current bandwidth usage in progress display

**Google/Read Topics:**

- [ ] "token bucket rate limiting Rust"
- [ ] "governor crate examples"
- [ ] "bandwidth throttling async streams"

**Self-Check:**

- [ ] Rate limits enforced accurately?
- [ ] Can change limits dynamically?
- [ ] No performance degradation when not limited?

**Book Content:** Chapter 7 - "Bandwidth Management"

---

## Phase 6: Production Hardening & Book Completion (Days 21-23)

### Task 24 — Comprehensive testing (Day 21)

**Priority:** CRITICAL - Production quality requirement

**Build Requirements:**

- [ ] Unit tests for each protocol implementation
- [ ] Integration tests for full download flows
- [ ] Mock servers for testing (HTTP, FTP, etc.)
- [ ] Property-based testing with `proptest` (URL parsing, range math)
- [ ] Concurrency stress tests (many workers, many downloads)
- [ ] Error injection tests (simulate network failures)
- [ ] CI/CD pipeline with GitHub Actions:
  - `cargo test`
  - `cargo clippy`
  - `cargo fmt --check`
  - `cargo audit` (security vulnerabilities)
- [ ] Coverage reports with `tarpaulin`

**Google/Read Topics:**

- [ ] "Rust testing best practices"
- [ ] "mockito or wiremock for HTTP mocking"
- [ ] "proptest examples"
- [ ] "GitHub Actions Rust workflow"

**Self-Check:**

- [ ] >70% code coverage?
- [ ] All protocols have integration tests?
- [ ] CI passes on every commit?

---

### Task 25 — Performance optimization (Day 22)

**Priority:** HIGH - Demonstrates production readiness

**Build Requirements:**

- [ ] Profile with `cargo flamegraph` or `perf`
- [ ] Optimize hot paths (identified via profiling)
- [ ] Consider `bytes` crate for zero-copy buffer management
- [ ] Use `tokio::io::BufReader` where appropriate
- [ ] Optimize state lock contention (minimize lock hold time)
- [ ] Consider lock-free structures (`crossbeam`, `DashMap`)
- [ ] Benchmark against `wget`, `curl` for HTTP downloads
- [ ] Memory profiling (ensure no leaks with long-running downloads)

**Google/Read Topics:**

- [ ] "Rust profiling flamegraph"
- [ ] "tokio performance tuning"
- [ ] "zero-copy buffer management bytes crate"
- [ ] "lock-free data structures Rust"

**Self-Check:**

- [ ] Performance competitive with wget/curl?
- [ ] No memory leaks in long-running tests?
- [ ] CPU usage reasonable under load?

---

### Task 26 — Documentation and book finalization (Day 23)

**Priority:** CRITICAL - The whole point!

**Build Requirements:**

- [ ] Complete all book chapters (1-7):
  - Chapter 1: HTTP foundations ✅
  - Chapter 2: Protocol abstraction & error handling
  - Chapter 3: FTP/SFTP implementation
  - Chapter 4: WebDAV and remote filesystems
  - Chapter 5: Cloud storage (S3)
  - Chapter 6: P2P protocols (BitTorrent, IPFS, HLS)
  - Chapter 7: Concurrency, control plane, production features
- [ ] Add architecture diagrams (use `mermaid` in mdbook)
- [ ] Code examples for each protocol
- [ ] Decision logs (why certain designs chosen)
- [ ] Performance benchmarks and results
- [ ] Troubleshooting guide
- [ ] API documentation with `cargo doc`
- [ ] README with quick start guide
- [ ] CONTRIBUTING guide
- [ ] License file (MIT or Apache-2.0)

**Self-Check:**

- [ ] Book builds without errors?
- [ ] All code examples tested and working?
- [ ] Clear learning path from chapter to chapter?
- [ ] Diagrams clarify architecture?

---

## Bonus Features for "The Download Book"

**If time permits or post-launch:**

- [ ] Web UI (React/Svelte frontend for control plane)
- [ ] Desktop app (Tauri wrapper around the core)
- [ ] Browser extension (download interceptor)
- [ ] MPEG-DASH support (companion to HLS)
- [ ] rsync protocol
- [ ] git:// protocol (clone as download)
- [ ] Database export protocols (PostgreSQL COPY, MySQL dump streaming)
- [ ] Metrics export (Prometheus endpoint)
- [ ] Distributed tracing (OpenTelemetry integration)
- [ ] Plugin system (dynamic protocol loading via `libloading`)

---

## Completed Features

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

---

## Book-Driven Development Workflow

### For Each Chapter:

1. **Implement** - Build the features listed in the chapter's tasks
2. **Test** - Verify functionality with real-world examples
3. **Document** - Write the chapter explaining:
   - What you built (code snippets, architecture diagrams)
   - Why you made specific decisions
   - What you learned (patterns, pitfalls, surprises)
   - What didn't work (honest about dead ends)
4. **Demo** - Create asciinema recording showing the feature in action
5. **Commit** - Tag the repo with chapter completion (e.g., `chapter-1-complete`)

### Daily Workflow:

**Morning:** Code (implement tasks for current chapter)
**Afternoon:** Code + test (verify with real protocols/servers)
**Evening:** Write (document what you built, capture learnings while fresh)

### Chapter Completion Criteria:

- ✅ All tasks for the chapter completed
- ✅ Code compiles and passes tests
- ✅ Feature works with real-world examples
- ✅ Chapter written with code examples
- ✅ Demo recorded (asciinema or screenshot)
- ✅ Committed and tagged

### Launch Checklist (After Chapter 7):

- [ ] All 7 chapters written and polished
- [ ] README.md updated with impressive feature list
- [ ] Asciinema demos for each protocol
- [ ] Architecture diagram in README
- [ ] Performance benchmarks (vs wget/curl if applicable)
- [ ] GitHub repo polished (issues disabled, clean commit history)
- [ ] Book deployed (GitHub Pages or similar)
- [ ] HN post drafted with killer title
- [ ] Tweet thread prepared
- [ ] r/rust crosspost ready

---

## Quick Reference: Chapter → Tasks

| Chapter | Focus | Tasks | Days | Protocols |
|---------|-------|-------|------|-----------|
| 1 | HTTP Foundation | 1-8 | 1-2 | HTTP/HTTPS |
| 2 | Daemon Architecture | 9-11, 16-17, 22 | 3-7 | *Foundation* |
| 3 | FTP | 12 | 8-9 | FTP/FTPS |
| 4 | BitTorrent | 19 | 10-15 | BitTorrent (tracker) |
| 5 | IPFS | 20 | 16-17 | IPFS |
| 6 | S3 | 15 | 18 | S3 + compatible |
| 7 | Retrospective | - | 19-20 | *Writing* |

**Total Protocols Implemented:** 5 (HTTP, FTP, BitTorrent, IPFS, S3)
**Total Lines of Code (estimated):** ~8,000-10,000
**Architecture Patterns:** Daemon, Protocol trait, Worker pools, REST API, State management, Structured logging

