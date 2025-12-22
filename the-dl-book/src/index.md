# Introduction

```admonish warning
This book is a work in progress (started December 18, 2024). You're watching me build this in real-time. Contact me at `mail [at] stonecharioteer.com` if you have questions or want to follow along.
```

Have you ever wanted to download something? Of course you have. But have you wanted to download _anything_? HTTP files, FTP archives, BitTorrent swarms, IPFS content hashes, S3 buckets—all through one tool?

That's what we're building. A download manager that doesn't care what protocol you throw at it.

## What This Book Is

This is not a Rust tutorial. The [Rust Book](https://doc.rust-lang.org/book/) exists for that. This is a book about building a complex, multi-protocol system in Rust while learning the hard parts of async programming, protocol implementation, and production architecture along the way.

I'm writing this as I build it. You'll see the decisions, the mistakes, the refactors. My mom always said the best way to learn is to write, so here we are.

## What We're Building

`dlm` - a download manager daemon that speaks:
- **HTTP/HTTPS** - The foundation (concurrent chunks, resume, range requests)
- **FTP/FTPS** - The 1971 protocol that refuses to die
- **BitTorrent** - P2P file sharing with tracker and DHT support
- **IPFS** - Content-addressed downloads
- **S3** - Cloud object storage (and S3-compatible services)

All coordinated through a daemon architecture with REST APIs, worker pools, job queues, and production-grade observability.

## The Journey

**Chapter 1** starts simple: download a file over HTTP. By the end, you'll have concurrent chunk downloads, progress tracking, pause/resume, and SHA256 verification.

**Chapter 2** transforms the CLI tool into a daemon. We build the architecture that everything else plugs into: the protocol abstraction layer, worker pools, state management, APIs, and logging infrastructure.

**Chapters 3-6** implement protocols. Each chapter adds a new protocol to the daemon, showing how the abstraction handles wildly different download mechanisms.

**Chapter 7** reflects on what worked, what didn't, and what I'd do differently.

## What You'll Learn

- Async Rust patterns with `tokio`
- Protocol implementation (from simple HTTP to complex P2P)
- Production architecture (daemons, APIs, observability)
- State management and concurrency
- Real-world trade-offs and decisions

## Prerequisites

You should know Rust basics. If you've read the Rust Book and written some async code, you're ready. If not, go do that first.

You should have `cargo` and a terminal. We're building a CLI/daemon tool, not a GUI.

## Before You Start

This is going to be messy. Protocols have sharp edges. Async Rust has footguns. BitTorrent will kick your ass. But by the end, you'll have built something genuinely complex and impressive.

Ready?

_Let's write a download manager, damn it._
