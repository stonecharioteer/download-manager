# Download Manager Implementation

## Summary

I've built a Rust-based download manager (`dlm`) with both blocking and async
implementations. The tool provides robust file downloading with progress
tracking, resume capability, and integrity verification.

**Core Features:**

- **Dual download modes**: Blocking (`reqwest::blocking`) and async (`tokio` +
  `reqwest` async) implementations, both accessible via CLI subcommands
- **Progress tracking**: Real-time download speed, bytes transferred, and
  elapsed time displayed using the `indicatif` library
- **Resume capability**: Partial download detection with HTTP Range header
  support, handling 206 Partial Content, 416 Range Not Satisfiable, and fallback
  for servers that don't support resuming
- **SHA256 verification**: Streaming hash calculation during download, including
  proper handling of existing partial files when resuming
- **Graceful interrupts**: Ctrl-C handling with proper cleanup and error
  reporting

**Technical Implementation:** The blocking version uses `std::fs` with manual
chunked reading. The async version leverages `tokio::select!` to race between
stream chunks and interrupt checks (every 500ms).

Download functions are now pure logic - they only update atomic counters
(`Arc<AtomicUsize>`, `Arc<AtomicU64>`) and return file paths. All UI concerns
(progress bar, printing, timing) are handled by the CLI layer. A separate
progress reporter task polls the atomics every 500ms, calculating speed and
updating the progress bar. This decoupling makes download functions testable and
reusable without UI dependencies.

Error handling uses the `anyhow` crate for clean, thread-safe error types,
replacing verbose `Box<dyn std::error::Error + Send + Sync + 'static>`
signatures.

**Performance:** The release build uses ~8-10% of a single CPU core during
downloads, with most overhead from SHA256 hashing rather than I/O operations.

**What's Left:** Task 3 requires exposing range downloads as explicit CLI flags
(`--range-start`, `--range-end`) to download arbitrary byte ranges to temp
files, enabling multi-range downloads that can be stitched together.

## Evolution

### Error Handling: From Raw Types to Anyhow

Initially, the codebase used `Box<dyn std::error::Error>` for error handling.
This worked fine for the blocking version, but when introducing async
functionality with `tokio::task::spawn_blocking`, we hit thread safety
requirements. The error type needed to be `Send + Sync + 'static` to cross
thread boundaries.

The first fix was explicitly adding these bounds:
`Box<dyn std::error::Error + Send + Sync + 'static>`. While this worked, it made
function signatures verbose and exposed the underlying complexity of thread-safe
error handling.

We then migrated to the `anyhow` crate, which provides `anyhow::Result<T>` as a
clean alias for `Result<T, anyhow::Error>`. Since `anyhow::Error` is
`Send + Sync + 'static` by design, this solved the threading issues while
dramatically improving code readability. The `anyhow!()` macro also made custom
error messages more ergonomic than manually boxing error strings.

### From Blocking to Async

The blocking implementation came first, using `std::fs::File`,
`reqwest::blocking::get()`, and manual chunked reading with `.read()`. Progress
tracking relied on checking `Instant::elapsed()` every iteration and updating
when a second had passed.

The async migration introduced several key differences:

- **File I/O**: Switched from `std::fs` to `tokio::fs` with `AsyncReadExt` and
  `AsyncWriteExt` traits
- **HTTP streaming**: Used `reqwest::get().await` with `.bytes_stream()` instead
  of manual `.read()` loops
- **Progress tracking**: Replaced manual time checks with
  `tokio::time::interval` and `tokio::select!` to race between stream chunks,
  progress updates, and interrupt checks

The `tokio::select!` pattern was particularly powerful - it allowed concurrent
polling of multiple async operations (stream.next(), progress ticks, interrupt
checks) without explicit threading. Whichever future completed first would
execute its arm, enabling responsive progress updates and interrupts even during
slow downloads.

### Decoupling Progress and Interrupt Logic

Originally, each download function created its own `ProgressBar` and set up its
own `ctrlc` handler. This led to duplication and made the code harder to
maintain.

The first refactoring moved both to `main()`, passing them as parameters to
download functions. However, this still left UI concerns (calling progress bar
methods, printing messages) embedded in download logic.

The second refactoring fully decoupled UI from logic:

- **Created `DownloadProgress` struct**: Contains `Arc<AtomicUsize>` for bytes
  downloaded, `Arc<AtomicU64>` for total bytes (0 = unknown), and
  `Arc<AtomicBool>` for interrupts
- **Download functions became pure**: They only update atomics via `store()`
  operations and return `PathBuf` results. No progress bar methods, no
  `println!` statements
- **Spawned progress reporter task**: A separate `tokio::spawn` task polls
  atomics every 500ms, calculates speed (`bytes / elapsed_secs`), and updates
  the progress bar with human-readable output
- **CLI layer handles everything user-facing**: Initial messages ("Downloading X
  to Y"), timing (captured before expensive hash calculation), final output with
  SHA256

This pattern enables:

- **Lock-free communication**: Atomics provide thread-safe sharing without
  `Mutex` overhead
- **Testability**: Download functions have no UI dependencies, making unit
  testing straightforward
- **Reusability**: Functions can be used in different contexts (CLI, library,
  API server) without modification
- **Responsiveness**: Progress updates never block download operations

The `Arc` clones are cheap (just incrementing a reference count), and atomic
operations use `Ordering::Relaxed` for progress (performance) and
`Ordering::SeqCst` for interrupts (correctness).

## Key Learnings and Design Decisions

### 1. `write_all` vs `write`

Early in development, we had to choose between `write()` and `write_all()`.
While `write()` can return after writing only some bytes (partial writes),
`write_all()` guarantees all bytes are written or returns an error. For a
download manager where data integrity is critical, `write_all()` was the clear
choice to avoid silent data loss.

### 2. Memory Optimization: Streaming vs Loading

When implementing SHA256 verification, we faced a choice: load the entire file
into memory or stream it in chunks. For a download manager that might handle
multi-gigabyte files, loading everything into memory would be prohibitive. We
chose chunked streaming with a configurable buffer size (default 65KB), allowing
the tool to handle arbitrarily large files with constant memory usage.

### 3. SHA256 Resume Challenge

When implementing resume functionality, initially only the newly downloaded
chunks were hashed, which produced incorrect hashes for resumed downloads. The
fix required hashing the existing partial file first (in chunks, maintaining the
streaming approach) before downloading and hashing new data. This ensured the
hash covered the entire file regardless of how many resume attempts occurred.

### 4. HTTP Status Code Nuances

Resume support revealed the complexity of HTTP Range requests:

- **206 Partial Content**: Server supports range requests and is sending the
  requested byte range
- **416 Range Not Satisfiable**: The requested range is invalid, often because
  the file is already complete
- **200 OK**: Server doesn't support ranges and is sending the entire file,
  making resume impossible

Handling all three cases gracefully (with helpful error messages for users) made
the tool more robust than simply checking for 206.

### 5. File Opening Optimization

The first resume implementation opened the file twice: once for writing new
chunks, and again for reading existing content to hash. This was inefficient and
violated the principle of doing each operation once. We refactored to: check
file size → hash existing content → reopen for appending. This reduced I/O
operations and made the control flow clearer.

### 6. Async Chunk Size Control

The `chunk_size` parameter controls buffer size in the blocking version (where
we manually call `.read()`), but has no effect on the async version using
`.bytes_stream()`. The async stream's chunk sizes are determined by the
underlying network implementation and HTTP library, not user code. This is a
distinction between manual control (blocking) and library-managed streaming
(async).

### 7. `tokio::select!` Usage

The `tokio::select!` macro has syntax similar to `match` but races futures. Key
points:

- Don't add `.await` in select arms - the macro awaits each future automatically
- Select picks the first ready future, not matching on values
- Background futures (like `interval.tick()`) never end, so you need explicit
  `None` checks on streams to break the loop
- Enables concurrent operations without explicit threading

### 8. Debug vs Release Performance

Testing revealed a 3x performance difference between debug and release builds:

- Debug build: ~30-33% CPU usage per core
- Release build: ~8-10% CPU usage per core

Most CPU time goes to SHA256 hashing (CPU-intensive crypto), not I/O operations.
This reinforced the importance of testing with `--release` for realistic
performance measurements, especially for compute-heavy operations like
cryptographic hashing.

### 9. Atomic Ordering Semantics

When using atomics for lock-free communication, memory ordering matters:

- **`Ordering::Relaxed`**: Used for progress counters (bytes downloaded, total
  bytes). These are "best effort" reads where slight delays or reorderings don't
  affect correctness. Progress bars showing slightly stale data is acceptable.
- **`Ordering::SeqCst`**: Used for interrupt flag. This ensures all threads see
  the same global order of operations. When Ctrl-C is pressed, we need
  immediate, consistent visibility across the progress reporter task and
  download loop.

The performance difference is negligible here (progress updates are already
throttled to 500ms), but understanding the semantics clarifies intent: relaxed
for metrics, sequential consistency for control flow.

### 10. Multi-Worker Concurrent Downloads

The implementation of Task 4 added support for downloading files using multiple
concurrent workers, each handling a separate byte range of the file.

**Key implementation details:**

- **Content-Length retrieval**: HEAD requests don't always return
  Content-Length. Solution: use GET request (headers only, body not consumed)
  which reliably provides the file size.
- **Range splitting**: Divide file into N equal ranges, with the last worker
  taking any remainder bytes to handle files not evenly divisible by worker
  count.
- **Worker spawning**: Each worker is a separate `tokio::spawn` task calling
  `download_range_async()` with its assigned byte range.
- **Part file naming**: `filename.part.start-end` format includes the byte range
  in the filename for clarity during debugging.
- **Merging**: Use `futures::join_all()` to wait for all workers. Results are
  collected in spawn order (no sorting needed), then parts are read sequentially
  and concatenated.
- **Cleanup**: Part files are removed after successful merge, with
  `--no-cleanup` flag available for debugging.

**Critical bug discovered and fixed:** Initially used `part_paths.sort()` to
order files before merging, but lexicographic sorting fails with multiple
workers. Example with 4 workers:

- Lexicographic order: `part.0-70778879`, `part.141557760-212336639`,
  `part.212336640-283115519`, `part.70778880-141557759`
- Correct order: `part.0-70778879`, `part.70778880-141557759`,
  `part.141557760-212336639`, `part.212336640-283115519`

The number "141557760" comes before "70778880" in string sorting because "1" <
"7". This resulted in incorrect file assembly and wrong SHA256 checksums.

**Solution**: Remove sorting entirely. `futures::join_all()` returns results in
the same order tasks were spawned, which already matches the byte range order.
This is an important property of `join_all` - it preserves task order.

**Testing results** (Alpine ISO, 270MB):

- 2 workers: ~6 seconds, SHA256 verified ✓
- 4 workers: ~7 seconds, SHA256 verified ✓

The slight time variation is due to network conditions rather than worker
overhead.

### 11. Multi-Worker Progress Visualization

The multi-worker concurrent download (Task 4) had no progress visualization -
users couldn't see which chunks were being downloaded or track individual worker
progress. The goal was to create a colored, per-chunk progress bar showing
real-time state: `[████░░██░░]`.

[![asciicast](https://asciinema.org/a/am62159WJwd5peJRbQDdTrQ9V.svg)](https://asciinema.org/a/am62159WJwd5peJRbQDdTrQ9V)

**Initial Challenge**: Progress bars need to know total size upfront, but we
only learn `content_length` after the initial HTTP request inside
`download_with_workers()`. Creating the progress bar inside the download
function would violate our established separation of concerns (download logic
shouldn't create UI).

**Solution - Content Length Helper**: Created `get_content_length(&url)` as a
separate function that makes an initial HTTP request just to retrieve
`content_length`. This allows the CLI to:

1. Call `get_content_length()` first
2. Create `ChunkProgressBar` with the correct size
3. Pass the progress bar to `download_with_workers()`

This maintains clean separation: the CLI handles all HTTP metadata requests for
UI purposes, while download functions focus purely on downloading.

**ChunkProgressBar Design**:

- **`ChunkState` enum**: `Pending`, `Downloading { worker_id }`, `Completed`,
  `Failed`
- **Thread-safe state tracking**: `Arc<Mutex<Vec<ChunkState>>>` for per-chunk
  states (needs mutation from multiple workers)
- **Lock-free byte tracking**: `Vec<Arc<AtomicUsize>>` for per-chunk bytes
  downloaded (high-frequency updates)
- **Colored rendering**: Using `colored` crate - green for completed,
  yellow/cyan/magenta for downloading (different colors per worker), gray for
  pending, red for failed

**Worker Integration**: Modified `download_range_async()` to accept `chunk_id`
and report progress:

- Sets state to `Downloading { worker_id: chunk_id }` when starting
- Calls `progress.update_chunk_bytes(chunk_id, downloaded)` on each received
  chunk
- Sets state to `Completed` on success or `Failed` on error/interrupt

Changed the loop to use `chunks_array.into_iter().enumerate()` instead of
`.iter()` to avoid lifetime issues with spawned tasks borrowing from a local
vector.

**CLI Rendering Management**: The CLI now:

1. Gets content length via helper
2. Creates `ChunkProgressBar`
3. Spawns background task that calls `progress.render()` every 100ms
4. Calls `download_with_workers()` with the progress bar
5. Aborts render task when download completes
6. Calls `progress.finish()` with completion message

This keeps rendering completely in the CLI layer while download functions only
update state.

**ProgressTracker Trait**: Created a common trait for both `DownloadProgress`
and `ChunkProgressBar`:

```rust
trait ProgressTracker: Send + Sync + Clone {
    fn interrupted(&self) -> Arc<AtomicBool>;
    fn update_progress(&self, bytes: usize);
    fn render(&self);
    fn finish(&self, msg: &str);
    fn abandon(&self, msg: &str);
}
```

Chunk-specific methods like `set_chunk_state()` and `update_chunk_bytes()` stay
in the `ChunkProgressBar` implementation, not in the trait. This maintains the
abstraction (trait handles common progress concerns) while allowing
type-specific functionality.

**Refactoring Hashing to Execute**: All three download methods
(`download_blocking`, `download_async_single`, `download_async_multi`) were
duplicating the same hashing logic at the end. Since hashing is common
post-processing (not part of the download implementation), we:

1. Changed all download methods to return `PathBuf` instead of `()`
2. Moved hashing logic into `execute()` after the match statement
3. Now hashing happens once in one place for all download types

**Benefits of This Approach**:

- Clean separation: Download functions update state, CLI handles UI lifecycle
- Testable: Progress tracking can be tested independently of download logic
- Extensible: Easy to add new progress types (JSON output, silent mode) by
  implementing the trait
- Type-safe: Trait-based polymorphism with zero runtime cost (monomorphization)
- Maintainable: Hashing logic isn't duplicated across three methods
