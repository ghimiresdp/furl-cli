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

    /// Number of threads, defaults to 8, maximum allowed 255
    #[arg(short, long, default_value_t = 8)]
    threads: u8,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let path = Path::new(&args.out);
    let threads = args.threads;

    if !path.exists() {
        println!("The destination path does not exist");
        exit(1);
    }

    // TODO: add extensive url pattern matcher
    let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
    if let Some(_) = re.captures(&args.url) {
        let mut downloader = Downloader::new(&args.url);
        if let Ok(_) = downloader.download(&args.out, Some(threads)).await {
            println!("Download Complete!")
        }
        return;
    }
    println!("Invalid URL provided");
    return;
}
