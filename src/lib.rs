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
//! ```
//! use furl_core::Downloader;
//!
//! async fn my_function() {
//!     let mut downloader = Downloader::new("https://example.com/files/file_1.txt".to_owned());
//!     downloader.download(&args.dest).await?;
//! }
//!
//! ```
pub mod engine;

pub use engine::Downloader;
