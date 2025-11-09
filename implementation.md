# Download Manager Implementation

## Summary

I've built a Rust-based download manager (`dlm`) with both blocking and async implementations. The tool provides robust file downloading with progress tracking, resume capability, and integrity verification.

**Core Features:**
- **Dual download modes**: Blocking (`reqwest::blocking`) and async (`tokio` + `reqwest` async) implementations, both accessible via CLI subcommands
- **Progress tracking**: Real-time download speed, bytes transferred, and elapsed time displayed using the `indicatif` library
- **Resume capability**: Partial download detection with HTTP Range header support, handling 206 Partial Content, 416 Range Not Satisfiable, and fallback for servers that don't support resuming
- **SHA256 verification**: Streaming hash calculation during download, including proper handling of existing partial files when resuming
- **Graceful interrupts**: Ctrl-C handling with proper cleanup and error reporting

**Technical Implementation:**
The blocking version uses `std::fs` with manual chunked reading and periodic progress updates. The async version leverages `tokio::select!` to race between stream chunks, progress updates (every 1s), and interrupt checks (every 500ms).

Both implementations share common UI components (progress bar, interrupt handling) through the main function, reducing duplication. The blocking function runs via `tokio::task::spawn_blocking` to avoid blocking the async runtime.

Error handling uses the `anyhow` crate for clean, thread-safe error types, replacing verbose `Box<dyn std::error::Error + Send + Sync + 'static>` signatures.

**Performance:**
The release build uses ~8-10% of a single CPU core during downloads, with most overhead from SHA256 hashing rather than I/O operations.

**What's Left:**
Task 3 requires exposing range downloads as explicit CLI flags (`--range-start`, `--range-end`) to download arbitrary byte ranges to temp files, enabling multi-range downloads that can be stitched together.

## Evolution

### Error Handling: From Raw Types to Anyhow

Initially, the codebase used `Box<dyn std::error::Error>` for error handling. This worked fine for the blocking version, but when introducing async functionality with `tokio::task::spawn_blocking`, we hit thread safety requirements. The error type needed to be `Send + Sync + 'static` to cross thread boundaries.

The first fix was explicitly adding these bounds: `Box<dyn std::error::Error + Send + Sync + 'static>`. While this worked, it made function signatures verbose and exposed the underlying complexity of thread-safe error handling.

We then migrated to the `anyhow` crate, which provides `anyhow::Result<T>` as a clean alias for `Result<T, anyhow::Error>`. Since `anyhow::Error` is `Send + Sync + 'static` by design, this solved the threading issues while dramatically improving code readability. The `anyhow!()` macro also made custom error messages more ergonomic than manually boxing error strings.

### From Blocking to Async

The blocking implementation came first, using `std::fs::File`, `reqwest::blocking::get()`, and manual chunked reading with `.read()`. Progress tracking relied on checking `Instant::elapsed()` every iteration and updating when a second had passed.

The async migration introduced several key differences:
- **File I/O**: Switched from `std::fs` to `tokio::fs` with `AsyncReadExt` and `AsyncWriteExt` traits
- **HTTP streaming**: Used `reqwest::get().await` with `.bytes_stream()` instead of manual `.read()` loops
- **Progress tracking**: Replaced manual time checks with `tokio::time::interval` and `tokio::select!` to race between stream chunks, progress updates, and interrupt checks

The `tokio::select!` pattern was particularly powerful - it allowed concurrent polling of multiple async operations (stream.next(), progress ticks, interrupt checks) without explicit threading. Whichever future completed first would execute its arm, enabling responsive progress updates and interrupts even during slow downloads.

### Centralizing UI Components

Originally, each download function created its own `ProgressBar` and set up its own `ctrlc` handler. This led to duplication and made the code harder to maintain.

We refactored by moving both to `main()`:
- **Progress bar**: Created once before the subcommand match, then passed as a parameter to both download functions
- **Interrupt system**: The `ctrlc::set_handler()` can only be called once globally, so we set it in `main()` with an `Arc<AtomicBool>`. The flag is then cloned and passed to whichever download function executes

This revealed an interesting constraint: `ProgressBar` needed to be `Send` to move into `spawn_blocking` for the blocking download. Fortunately, `indicatif`'s implementation is already thread-safe, so this compiled without issues.

The refactoring eliminated code duplication, clarified the separation between UI concerns (main) and download logic (functions), and ensured consistent behavior across both blocking and async modes.

## Key Learnings and Design Decisions

### 1. `write_all` vs `write`

Early in development, we had to choose between `write()` and `write_all()`. While `write()` can return after writing only some bytes (partial writes), `write_all()` guarantees all bytes are written or returns an error. For a download manager where data integrity is critical, `write_all()` was the clear choice to avoid silent data loss.

### 2. Memory Optimization: Streaming vs Loading

When implementing SHA256 verification, we faced a choice: load the entire file into memory or stream it in chunks. For a download manager that might handle multi-gigabyte files, loading everything into memory would be prohibitive. We chose chunked streaming with a configurable buffer size (default 65KB), allowing the tool to handle arbitrarily large files with constant memory usage.

### 3. SHA256 Resume Challenge

When implementing resume functionality, initially only the newly downloaded chunks were hashed, which produced incorrect hashes for resumed downloads. The fix required hashing the existing partial file first (in chunks, maintaining the streaming approach) before downloading and hashing new data. This ensured the hash covered the entire file regardless of how many resume attempts occurred.

### 4. HTTP Status Code Nuances

Resume support revealed the complexity of HTTP Range requests:
- **206 Partial Content**: Server supports range requests and is sending the requested byte range
- **416 Range Not Satisfiable**: The requested range is invalid, often because the file is already complete
- **200 OK**: Server doesn't support ranges and is sending the entire file, making resume impossible

Handling all three cases gracefully (with helpful error messages for users) made the tool more robust than simply checking for 206.

### 5. File Opening Optimization

The first resume implementation opened the file twice: once for writing new chunks, and again for reading existing content to hash. This was inefficient and violated the principle of doing each operation once. We refactored to: check file size → hash existing content → reopen for appending. This reduced I/O operations and made the control flow clearer.

### 6. Async Chunk Size Control

The `chunk_size` parameter controls buffer size in the blocking version (where we manually call `.read()`), but has no effect on the async version using `.bytes_stream()`. The async stream's chunk sizes are determined by the underlying network implementation and HTTP library, not user code. This is a distinction between manual control (blocking) and library-managed streaming (async).

### 7. `tokio::select!` Usage

The `tokio::select!` macro has syntax similar to `match` but races futures. Key points:
- Don't add `.await` in select arms - the macro awaits each future automatically
- Select picks the first ready future, not matching on values
- Background futures (like `interval.tick()`) never end, so you need explicit `None` checks on streams to break the loop
- Enables concurrent operations without explicit threading

### 8. Debug vs Release Performance

Testing revealed a 3x performance difference between debug and release builds:
- Debug build: ~30-33% CPU usage per core
- Release build: ~8-10% CPU usage per core

Most CPU time goes to SHA256 hashing (CPU-intensive crypto), not I/O operations. This reinforced the importance of testing with `--release` for realistic performance measurements, especially for compute-heavy operations like cryptographic hashing.
