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
use furl_core::engine::DownloadConfig;
use furl_core::{Downloader, FurlCliArgs, GraphicalProgressReporter};
use regex::Regex;
use std::process::exit;

use std::path::Path;

#[tokio::main]
async fn main() {
    let args = FurlCliArgs::parse();
    let path = Path::new(&args.out);
    let threads = args.threads;
    let filename = args.filename;
    let chunk_size = args.chunksize;

    if !path.exists() {
        println!("The destination path does not exist");
        exit(1);
    }

    // TODO: add extensive url pattern matcher
    let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
    if re.captures(&args.url).is_some() {
        // use config
        let download_config =
            DownloadConfig::new().set_max_chunk_size(chunk_size as u64 * 1024 * 1024);

        let mut downloader = Downloader::new(&args.url)
            .with_config(download_config)
            .with_reporter(GraphicalProgressReporter::new());
        if downloader
            .download(&args.out, filename, Some(threads))
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
