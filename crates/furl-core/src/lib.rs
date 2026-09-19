//! # furl_core
//!
//! fURL core package includes different structs and functions that can be used
//! in your package for downloading files without using `furl-cli`.
//!
//! ## installation
//!
//! furl-core has no default features. Enable `progress` if you want the
//! built-in graphical progress reporter, or `serde` if you want
//! `DownloadConfig` to be (de)serializable.
//!
//! You can add `furl-core` to your project by adding it to your `cargo.toml` file.
//! or you can use the command line to add it to your project.
//!
//! ### Using `cargo add`
//! ```bash
//! cargo add furl-core
//! ```
//!
//! ### `cargo.toml`
//! ```toml
//! [dependencies]
//! furl-core = { version = "0.10.0" }
//!
//! # example async library for async operations
//! tokio = { version = "1.52.3", features = ["rt-multi-thread", "macros"] }
//! ```
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
//! furl-core = { version = "*", features = ["progress"] }
//!
//! ```
pub mod core;
pub mod features;

pub use core::config::DownloadConfig;
pub use core::engine::{Downloader, ProgressReporter};

pub use features::*;
