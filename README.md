# Download Manager (dlm)

A Rust learning project exploring async programming, concurrency, and systems
design through building a feature-rich file download manager.

## About This Project

This is a learning exercise where I'm relearning Rust by building a
progressively complex download manager from scratch. The project follows a
structured learning path, starting with basic blocking downloads and evolving
into a concurrent, multi-worker system with real-time progress visualization.

**Focus Areas:**

- Async programming with `tokio` and `reqwest`
- Concurrent worker coordination
- Thread-safe state management (`Arc`, `Mutex`, atomics)
- CLI design with `clap`
- Progress visualization with `indicatif`
- Error handling patterns (`anyhow`, `thiserror`)
- Systems design and architecture evolution

## Current Status

**Completed:**

- ✅ Task 1: Basic blocking single-file download with progress
- ✅ Task 2: Async implementation with `tokio`
- ✅ Task 3: HTTP Range header support for resumable downloads
- ✅ Task 4: Multi-worker concurrent downloads with chunk splitting
- ✅ Task 5: Color-coded chunk progress visualization

**In Progress:**

- 🔨 Task 6: Pause and resume with state persistence

See [FEATURES.md](FEATURES.md) for the complete roadmap.

## Demo

Multi-worker download with live progress visualization:

[![asciicast](https://asciinema.org/a/am62159WJwd5peJRbQDdTrQ9V.svg)](https://asciinema.org/a/am62159WJwd5peJRbQDdTrQ9V)

## Features

- **Multiple download modes**: Blocking, async single-worker, and async
  multi-worker
- **Resume capability**: Automatically resume interrupted downloads
- **Progress tracking**: Real-time visualization with download speed and ETA
- **Multi-worker visualization**: Color-coded chunk progress for concurrent
  downloads
- **SHA256 verification**: Streaming hash calculation for file integrity
- **Graceful interrupts**: Clean Ctrl-C handling with proper cleanup

## Usage

```bash
# Single-threaded async download
cargo run -- download-async <url>

# Multi-worker concurrent download (4 workers)
cargo run -- download-async --workers 4 <url>

# Blocking download
cargo run -- download-blocking <url>

# Options
cargo run -- download-async --workers 4 \
  --target-directory ./downloads \
  --chunk-size 65536 \
  --resume \
  --overwrite \
  <url>
```

## Implementation Notes

The project emphasizes learning through iteration. Each task builds on the
previous one, introducing new Rust concepts and architectural patterns. Detailed
design decisions, evolution notes, and key learnings are documented in
[implementation.md](implementation.md).

**Key architectural decisions:**

- Pure download functions that only update state (no UI concerns)
- CLI layer manages all progress rendering and user interaction
- Lock-free progress tracking with atomics for performance
- Trait-based abstraction for different progress visualizations
- Clean separation between blocking and async implementations

## Formatting

Markdown sources (including the chapter drafts under `the-dl-book/src`) follow a
shared Prettier setup so prose stays wrapped at 80 characters. Install Prettier
globally with `npm i -g prettier` so the `just` recipes can call the binary,
then run `just format-md` (or `just check-md`) to execute
`prettier --write '**/*.md'`/`prettier --check '**/*.md'`. Generated output
directories are skipped via `.prettierrc.cjs` and `.prettierignore`.

## Building

```bash
# Development build
cargo build

# Release build (recommended for actual downloads)
cargo build --release

# Run
cargo run -- download-async --workers 4 <url>
```

## Learning Resources

This project follows a self-directed learning path documented in
[FEATURES.md](FEATURES.md). Each task includes:

- Build goals
- Relevant reading/search queries
- Self-check criteria

The [implementation.md](implementation.md) document chronicles the evolution of
the codebase, design decisions, and lessons learned at each stage.

## License

This is a personal learning project. Feel free to use it as reference for your
own Rust learning journey.
