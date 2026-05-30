//! # furl_core
//!
//! fURL core package includes different structs and functions that can be used
//! in your package for downloading files without using `furl-cli`.
//!
//! ## Example Usecase
//!
//! since the package implements multi-threaded downloading, it is expected to
//! be used inside an async function.
//!
//! please check more about async rust
//! [here (Async Book)](https://rust-lang.github.io/async-book/).
//!
//! or check [tokio](https://crates.io/crates/tokio) package for more detailed implementation.
//!
//! ```rust
//! use furl_core::{Downloader};
//!
//! #[tokio::main]
//! async fn main() {
//!     let url = "https://raw.githubusercontent.com/ghimiresdp/furl-cli/refs/heads/main/res/images/example.png";
//!     let mut downloader = Downloader::new(url);
//!     if let Ok(_) = downloader.download("./downloads/", None, Some(4)).await {
//!         println!("Download completed successfully!");
//!     } else {
//!         println!("Download failed.");
//!     }
//! }
//! ```
//!
//! If you want to use the progress reporter, you can use the `GraphicalProgressReporter` from the `progress` feature.
//! for that you need to enable the `progress` feature in your `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! furl-core = { version = "0.8.0", features = ["progress"] }
//!
//! ```
pub mod engine;
pub mod features;

pub use engine::{DownloadConfig, Downloader, ProgressReporter};

// re-exporting features for easier access
#[cfg(feature = "progress")]
pub use features::progress::GraphicalProgressReporter;

#[cfg(feature = "cli")]
pub use features::cli::FurlCliArgs;
