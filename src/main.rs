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
    let config = match config::FurlConfig::init() {
        Ok(config) => config,
        Err(_) => {
            println!("Config does not exist, using default config");
            config::FurlConfig::default()
        }
    };
    let cli = cli::parse();
    return cli.execute(config).await;
}
