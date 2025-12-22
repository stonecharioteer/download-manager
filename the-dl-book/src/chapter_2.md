# Gone, gone the form of man, rise the Daemon

```admonish abstract title="Chapter Guide"
**What you'll build:** Transform the CLI tool into a daemon with protocol abstraction, worker pools, job queue, state management, REST API, and structured logging.

**What you'll learn:** Daemon architecture, `async_trait` for protocol abstraction, `Arc<RwLock<_>>` for shared state, Axum for REST APIs, `tracing` for observability, production error handling with `thiserror`.

**End state:** Tasks 9, 10, 11, 16, 17, and 22 completed. The daemon is running with a protocol registry, and the CLI becomes a thin client talking to the daemon via REST API. All future protocols plug into this architecture.
```

