# Embedded Minimal Example

This example demonstrates how to embed `furl-core` in a Rust application with visual progress reporting.

## Running This Example

### Inside the Repository

If you cloned the entire `furl-cli` repository, you can run:

```bash
cargo run -p embedded-minimal
```

This uses the local path dependency to `furl-core`, so you'll always be working with the latest code from the repository.

### Standalone / Outside the Repository

To use this example as a standalone project or copy it elsewhere, update `Cargo.toml`:

**Current (uses crates.io):**

```toml
furl-core = { version = "0.9.0", features = ["progress"] }
```

If you want to use the local version while developing, change to:

```toml
furl-core = { path = "../../crates/furl-core", features = ["progress"] }
```

Then run:

```bash
cargo run
```

## What This Example Shows

- Creating a `Downloader` instance with a URL
- Setting up `GraphicalProgressReporter` for visual progress feedback
- Configuring chunk size for parallel downloads (5 MB in this example)
- Running the download asynchronously with 4 worker threads
- Basic error handling for download operations

## Dependencies

- `furl-core` — The multithreaded download engine (with `progress` feature for visual bars)
- `tokio` — Async runtime for concurrent operations

## Customization

You can modify the example by:

- Changing the URL to download different files
- Adjusting `set_max_chunk_size()` for different chunk sizes
- Modifying the thread count in `download()` for more/fewer parallel workers
- Implementing your own `ProgressReporter` trait for custom progress handling
