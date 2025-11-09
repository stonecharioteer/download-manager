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
- [ ] Are you awaiting everything correctly (no blocking code)?
- [ ] Does it use <100% CPU while downloading?
- [ ] Can you interrupt with Ctrl+C safely (graceful shutdown)?

---

### Task 3 — Ranged + chunked download skeleton

**Build Requirements:**
- [ ] Accept a `--range-start` flag
- [ ] Accept a `--range-end` flag
- [ ] Use `Range` header in request
- [ ] Download only that part of the file to a temp file

**Google/Read Topics:**
- [ ] "reqwest set header Range bytes example"
- [ ] "HTTP Range header format"
- [ ] "fs::OpenOptions append/truncate mode"

**Self-Check:**
- [ ] Does server return `206 Partial Content`?
- [ ] Is downloaded chunk size correct?
- [ ] Can multiple non-overlapping ranges be stitched together with `cat`?

---

## Bonus Features Implemented (Beyond README Requirements)

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
