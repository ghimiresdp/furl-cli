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
//! use furl_core::Downloader;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let mut downloader = Downloader::new("https://example.com/files/file_1.txt");
//!     // download into the current directory using default thread count and custom filename
//!     downloader.download(".", Some("file_123.txt".to_string()), None).await?;
//!     Ok(())
//! }
//! ```
pub mod engine;

pub use engine::Downloader;
