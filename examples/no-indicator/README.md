# No-Indicator Example

This example demonstrates how to use `furl-core` for headless downloads without any visual progress reporting — ideal for servers, background jobs, or headless applications.

## Running This Example

### Inside the Repository

If you cloned the entire `furl-cli` repository, you can run:

```bash
cargo run -p no-indicator
```

This uses the local path dependency to `furl-core`, so you'll always be working with the latest code from the repository.

### Standalone / Outside the Repository

To use this example as a standalone project or copy it elsewhere, update `Cargo.toml`:

**Current (uses crates.io):**

```toml
furl-core = { version = "0.9.0" }
```

If you want to use the local version while developing, change to:

```toml
furl-core = { path = "../../crates/furl-core" }
```

Then run:

```bash
cargo run
```

## What This Example Shows

- Creating a `Downloader` with **no progress reporter** (silent operation)
- Using `NoopReporter` implicitly for headless/background downloads
- Setting up chunk size configuration for parallel downloads
- Running downloads without any terminal output or visual feedback
- Minimal dependencies — only `furl-core` and `tokio`

## Use Cases

This pattern is ideal for:

- **Server-side downloads** — background download jobs without UI
- **Batch processing** — downloading multiple files silently
- **Game engines** — asset downloads without terminal output
- **Embedded systems** — minimal logging and no progress bars
- **Testing** — simple, reproducible download behavior

## Customization

You can extend this example by:

- Implementing your own `ProgressReporter` trait for custom logging or metrics
- Adding error handling and retry logic
- Logging download stats after completion
- Integrating with a metrics system or monitoring service
- Adjusting thread count for your specific use case

## Dependencies

- `furl-core` — The multithreaded download engine (without the `progress` feature for lean dependencies)
- `tokio` — Async runtime for concurrent operations
