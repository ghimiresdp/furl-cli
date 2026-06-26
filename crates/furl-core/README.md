# furl-core

[![CI](https://github.com/ghimiresdp/furl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/ghimiresdp/furl-cli/actions/workflows/ci.yml)

A fast, multithreaded downloader library for Rust — the core engine behind the
`furl-cli` binary.

furl-core contains the downloading logic (chunking, concurrent workers, and a
progress reporting abstraction) as a small, dependency-light library that you
can embed in your own async applications. The binary CLI and UI concerns live
in `furl-cli`; keeping those responsibilities separate makes the core crate
easier to reuse and keeps dependencies optional via features.

## Key features

- Smart parallel downloads: splits large files into chunks and downloads them
  concurrently.
- Small-file optimization: smaller files are downloaded without spawning worker
  threads.
- Progress abstraction: a `ProgressReporter` trait lets consumers plug in any
  reporting implementation.
- Optional graphical progress: the `progress` feature provides an
  `indicatif`-based `GraphicalProgressReporter`.
- Configurable chunk size via `DownloadConfig`.
- Built with `tokio` + `reqwest` for async, streaming downloads.

## Crate version

This crate is published as `furl-core` (see `Cargo.toml`). For local development
inside this workspace, use the path dependency. For published releases, use the
crate version on crates.io.

## Quick start (library mode)

Add `furl-core` to your `Cargo.toml`.

Using crates.io (recommended for applications):

```toml
[dependencies]
# Replace with the latest version on crates.io
furl-core = "0.9.0-alpha.1"

# enable progress bars if you want the graphical reporter
furl-core = { version = "0.9.0-alpha.1", features = ["progress"] }
```

Async example (using `tokio`):

```rust
// GraphicalProgressReporter requires the `progress` feature
use furl_core::{DownloadConfig, Downloader, GraphicalProgressReporter};

#[tokio::main]
async fn main() {
    let url = "https://example.com/path/to/file.png";

    // Optional: tune chunk size (default is 10 MB)
    let chunk_size = 5 * 1024 * 1024; // 5 MB
    let config = DownloadConfig::default().set_max_chunk_size(chunk_size);

    // Create a downloader and attach a progress reporter (optional)
    let mut downloader = Downloader::new(url)
        .with_reporter(GraphicalProgressReporter::new())
        .with_config(config);

    // `download` is async.
    if downloader.download(".", None, Some(4)).await.is_ok() {
        println!("Download completed successfully!");
    } else {
        println!("Download failed.");
    }
}
```

See the workspace examples for runnable samples:
<https://github.com/ghimiresdp/furl-cli/tree/main/examples>

## API overview

This section covers the main public types and methods you will use.

- `Downloader` — primary entry point
  - `Downloader::new<S: Into<String>>(url: S) -> Downloader`
  - `with_config(self, config: DownloadConfig) -> Self`
  - `with_reporter<R: ProgressReporter + Send + Sync + 'static>(self, reporter: R) -> Self`
  - `async fn download(&mut self, path: &str, filename: Option<String>, threads: Option<u8>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>`

- `DownloadConfig` — configuration for chunking
  - `DownloadConfig::new()` / `Default`
  - `set_max_chunk_size(self, size: u64) -> Self`

- `ProgressReporter` — trait for progress callbacks
  - `fn on_start(&self, chunk_index: usize, total_bytes: u64)`
  - `fn on_progress(&self, chunk_index: usize, bytes_downloaded: u64)`
  - `fn on_finish(&self, chunk_index: usize)`

- Feature-provided type (enable `progress`)
  - `GraphicalProgressReporter` — an `indicatif`-backed implementation of `ProgressReporter`

Notes:

- If the server provides a `Content-Range` header, the library will determine
  file size and split the download into chunks for concurrent workers.
- If file size cannot be determined, the library will fall back to downloading
  without threads and without chunked progress (a simple spinner/ticker is used
  if you attach a reporter).
- Files smaller than 1 MB are downloaded without spawning worker threads.

## Examples in this repository

- `examples/embedded-minimal` — embedding `furl-core` inside a small binary with
  `GraphicalProgressReporter` enabled. (See `examples/embedded-minimal/src/main.rs`)
- `examples/no-indicator` — example that runs without the graphical progress feature.

Run an example from the workspace root:

```bash
# run the embedded example
cargo run -p embedded-minimal

# run the test suite for the core crate
cargo test -p furl-core
```

## Design notes and roadmap

- The project separates concerns: `furl-core` implements the download engine,
  while `furl-cli` provides the command-line interface and user-facing UX.
  This split keeps `furl-core` reusable with minimal optional dependencies.
- Planned improvements (non-exhaustive): resume/checkpoint support, proxy and
  auth options, finer-grained configuration.

For architecture decisions, see `docs/adr/` in the repository.

## Contributing

Contributions are welcome. If you want to modify the core downloader, please
open an issue first to discuss the change. See the repository `CONTRIBUTING.md`
for guidelines.

## License

`furl-core` is licensed under the Apache License 2.0. See the `LICENSE` file
at the repository root for details.
