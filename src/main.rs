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
use furl_core::Downloader;
use regex::Regex;
use std::process::exit;

use std::path::Path;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct CliArgs {
    #[arg()]
    url: String,

    #[arg(default_value_t = String::from("."))]
    dest: String,
    // TODO: add arguments for number of threads, etc.
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let path = Path::new(&args.dest);
    if !path.exists() {
        println!("The destination path does not exist");
        exit(1);
    }

    // TODO: add extensive url pattern matcher
    let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
    if let Some(_) = re.captures(&args.url) {
        let mut downloader = Downloader::new(&args.url);
        if let Ok(_) = downloader.download(&args.dest).await {
            println!("Download Complete!")
        }
    } else {
        println!("Invalid Url parameter provided");
        exit(1);
    }
}
