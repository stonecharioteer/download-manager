# Introduction

Have you ever wanted to download something? No? You're not me? Well then. I don't know how to help you.
But if you do, and if you also wanted to download something using code, then you're in luck. If you're looking
for humour, satire and possibly catastrophe, you are in the right place. Also, of course, if you're looking
for some Rust.

What _is_ this book, you ask? It's not a Rust tutorial. If you want that, you should go to the Rust book. Trust me,
any attempt to learn Rust by following _this_ book is futile. I will not teach you Rust. Do _I_ even know Rust? I have no clue. Let's see by the end of this ludicrous exercise.

I wanted to write a download manager to understand Rust better. It had been a hot minute since I'd written Rust, and a friend
commented that I should try writing a download manager. I also wanted to write a bittorrent client. And I was also investigating
how data is transmitted from databases and what I could learn from that.

You see, I am curious to a fault and thought perhaps I should write something like an _overengineered_ download manager.
And then, I realized I wanted to write something that can download _anything_. So that's the goal of this "book". My mom always
says that the best way to learn something is to write, and that's what I grew up on so we're doing this. I'm writing a book
on how to build the most over-engineered download manager you can find. And what are we going to download, you ask?

That's the fun part.

## Laundry List

### HTTP/HTTPS

- HTTP/HTTPS downloads already work via
  [`stonecharioteer/download-manager`](https://github.com/stonecharioteer/download-manager),
  up to handling redirects, range requests, proxies, and other HTTP plumbing.
  Downloading a publicly available ISO is already in play, so the learning focus
  can now shift elsewhere.
- Rust concepts already explored: `tokio` async runtime, `anyhow` error
  handling, async/await, and pattern matching on response states.

### Protocol-rich transport

- FTP (active/passive): experiment with control/data channel handling and
  concurrency.
- FTPS (explicit/implicit TLS): layer in TLS negotiation.
- SFTP (over SSH): learn async SSH clients and session semantics.
- Rust concepts to stretch: low-level async I/O traits, `tokio`
  `TcpStream`/`AsyncRead`/`AsyncWrite`, `tokio-rustls` or `rustls` for TLS, and
  designing state machines via enums/traits to handle multi-channel
  conversations.

### Remote filesystems

- WebDAV endpoints for HTTP-based file tree traversal.
- SMB / CIFS network shares with their own authentication and negotiation.
- Treating NFS mounts as remote sources observes Unix permissions and blocking
  I/O.
- Rust concepts to try: bridging blocking FFI or `std::fs` calls with async
  runtimes via `spawn_blocking`, `tokio::fs`, dealing with
  `std::os::unix::fs::MetadataExt`, and writing ergonomic wrappers around unsafe
  or system-level APIs.

### Cloud object storage

- S3 pre-signed URLs and the S3 API (List/Get/Head) to understand pagination and
  XML responses.
- GCS / Azure Blob equivalents and generic S3-compatible systems (MinIO, Wasabi)
  to cover RESTful but subtly different APIs.
- Rust flavor: parsing XML/JSON with `serde`, modeling paginated flows, caching
  credentials, using `reqwest`/`hyper` abstractions for signed headers, and
  composing retries with `tokio::time`.

### Swarm & distributed

- BitTorrent (.torrent files) and magnet links for P2P piece scheduling.
- DHT + trackers (HTTP/UDP) plus uTP / TCP transports for routing and transport
  experimentation.
- Private / tracker-only torrents to understand permissioned swarms.
- IPFS (ipfs://, ipns://) and maybe a custom “LAN swarm” protocol for
  content-addressed discovery.
- Rust concepts to embrace: concurrency-safe graph/state tracking via
  `Arc<Mutex<_>>` or `tokio::sync` primitives, implementing protocols with
  `async_trait`, low-level UDP/TCP handling, and using `bytes`/`tokio-util` for
  framing/parsing.

### Streaming-ish workloads

- HLS (m3u8 playlists – sequential TS/fragment downloads) and MPEG-DASH (MPD
  manifest) to model playlist parsing and chunking.
- Progressive HTTP media, treated as large files but with long-running transfers
  and resume concerns.
- Rust ideas: parsing playlists with `nom` or `serde`, modeling sequential
  downloads with `Stream` combinators, scheduling background tasks with
  `tokio::spawn`, and building resume/timeout logic with `tokio::time::sleep`.

These learning tracks represent the next big push beyond the HTTP work that
already ships in the base download manager.

## Before you start

We're downloading _everything_.

_Let's write a download-manager, damn-it._
