# Download Manager

This is a learning exercise for Rust.

### Phase 1

**Task 1 — Basic blocking single-file download (MVP)**
(1) Build: A CLI tool `dlm` that downloads the Alpine ISO using `reqwest::blocking` and saves it to disk. Show progress via simple byte counter printed every second.
(2) Google/read:

- “reqwest blocking download file example”
- “Rust clap derive subcommand example”
- “std::fs::File write_all vs write”
  (3) Self-check:
- Does it resume cleanly on network failure?
- Does progress update correctly in bytes/sec?
- Is output file identical in size to `Content-Length`?

---

**Task 2 — Switch to async + tokio**
(1) Build: Convert the blocking downloader into an async version using `reqwest` + `tokio::fs`. Keep same progress reporting logic but via `tokio::time::interval`.
(2) Google/read:

- “reqwest async stream response body”
- “tokio::io::AsyncWriteExt example”
- “tokio::time::interval usage”
  (3) Self-check:
- Are you awaiting everything correctly (no blocking code)?
- Does it use <100% CPU while downloading?
- Can you interrupt with Ctrl+C safely (graceful shutdown)?

---

**Task 3 — Ranged + chunked download skeleton**
(1) Build: Modify the downloader to accept a `--range-start` and `--range-end` flag. Use `Range` header in request. Just download that part of the file to a temp file.
(2) Google/read:

- “reqwest set header Range bytes example”
- “HTTP Range header format”
- “fs::OpenOptions append/truncate mode”
  (3) Self-check:
- Does server return `206 Partial Content`?
- Is downloaded chunk size correct?
- Can multiple non-overlapping ranges be stitched together with `cat`?
