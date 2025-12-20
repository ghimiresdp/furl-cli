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
#[command(version, about, long_about=None, arg_required_else_help(true))]
struct CliArgs {
    /// url to download the file from
    #[arg()]
    url: String,

    /// output directory, defaults to the current directory
    #[arg(short, long, default_value_t = String::from("."))]
    out: String,
    // TODO: add arguments for number of threads, etc.
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let path = Path::new(&args.out);
    if !path.exists() {
        println!("The destination path does not exist");
        exit(1);
    }

    // TODO: add extensive url pattern matcher
    let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
    if let Some(_) = re.captures(&args.url) {
        let mut downloader = Downloader::new(&args.url);
        if let Ok(_) = downloader.download(&args.out).await {
            println!("Download Complete!")
        }
    } else {
        println!("Invalid Url parameter provided");
        exit(1);
    }
}
