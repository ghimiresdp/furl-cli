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

use clap::Parser;
use furl_cli::config::{self, load_config};
use furl_cli::{FurlCliArgs, FurlCommand};
use furl_core::{Downloader, GraphicalProgressReporter};
use regex::Regex;
use std::process::exit;

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let args = FurlCliArgs::parse();

    if let Some(FurlCommand::Config { key, value, reset }) = args.command {
        config::handle(key, value, reset);
        return;
    }

    let Some(url) = args.download.url else {
        eprintln!("Error: a URL is required");
        exit(1);
    };

    // config file values, overridden by any CLI args the user passed
    let mut config = load_config();
    if let Some(out) = args.download.out {
        config = config.set_download_dir(PathBuf::from(out));
    }
    if let Some(threads) = args.download.threads {
        config = config.set_threads(threads);
    }
    if let Some(chunksize) = args.download.chunksize {
        config = config.set_max_chunk_size(chunksize);
    }

    let filename = args.download.filename;

    if !config.download_dir.exists() {
        println!("The destination path does not exist");
        exit(1);
    }

    // TODO: add extensive url pattern matcher
    let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
    if re.captures(&url).is_some() {
        let out = config.download_dir.to_string_lossy().into_owned();
        let threads = config.threads;

        let mut downloader = Downloader::new(&url)
            .with_config(config)
            .with_reporter(GraphicalProgressReporter::new());
        if downloader
            .download(&out, filename, Some(threads))
            .await
            .is_ok()
        {
            println!("Download Complete!")
        }
        return;
    }
    println!("Invalid URL provided");
    return;
}
