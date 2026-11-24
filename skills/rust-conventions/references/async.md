# Async

Read this before writing or changing async code, spawning background tasks, or mixing sync and async I/O.

- Use `tokio` as the async runtime. Do not mix runtimes.
- Never call `.block_on()` inside an async context.
- Use `tokio::spawn` for background tasks; propagate `JoinHandle` errors.
- Prefer `tokio::fs` over `std::fs` in async code.
- Keep `async` boundaries at the edges — pure computation should be synchronous.
