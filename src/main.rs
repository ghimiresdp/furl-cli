//! # `furl-cli`
//!
//! A fast, multithreaded CLI downloader built in Rust.
//!
//! ## Usage
//!
//! ```bash
//! furl [URL]
//! ```
//!

use furl_core::{cli, config};

#[tokio::main]
async fn main() {
    if let Ok(config) = config::FurlConfig::init() {
        let cli = cli::parse();
        return cli.execute(Some(config)).await;
    }

    println!("Config path does not exist. Are you on supported OS?");
    return;
}
