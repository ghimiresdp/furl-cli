use std::path::Path;

use clap::{Parser, Subcommand};
use regex::Regex;

use crate::{Downloader, config::FurlConfig};

#[derive(Debug, Subcommand)]
pub enum FurlSubCommands {
    config,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about=None, arg_required_else_help(true), subcommand_negates_reqs = true)]
pub struct Cli {
    /// url to download the file from
    #[arg()]
    pub url: Option<String>,

    /// output directory, defaults to the current directory
    #[arg(short, long, default_value_t = String::from("."))]
    pub out: String,

    /// Number of threads, defaults to 8, maximum allowed 255
    #[arg(short, long, default_value_t = 8)]
    pub threads: u8,

    #[command(subcommand)]
    pub subcommand: Option<FurlSubCommands>,
}

impl FurlSubCommands {
    pub async fn execute(&self, config: FurlConfig) {
        match self {
            FurlSubCommands::config => {
                if let Ok(config) = FurlConfig::load() {
                    println!("Config: {:?}", config);
                } else {
                    println!("⛔⛔ error loading configuration configuration");
                }
            }
        }
    }
}

impl Cli {
    pub async fn execute(&self, config: FurlConfig) {
        if let Some(ref cmd) = self.subcommand {
            return cmd.execute(config).await;
        } else {
            // handle download actions
            let path = Path::new(&config.download_dir);
            let threads = config.threads;

            if !path.exists() {
                println!("The destination path does not exist");
                return;
            }

            if let Some(url) = &self.url {
                // TODO: add extensive url pattern matcher
                let re = Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap();
                if let Some(_) = re.captures(url) {
                    let mut downloader = Downloader::new(url);
                    if let Ok(_) = downloader.download(&self.out, Some(threads), config).await {
                        println!("Download Complete!")
                    }
                    return;
                }
                println!("Invalid URL provided");
                return;
            }
            return;
        }
    }
}

pub fn parse() -> Cli {
    Cli::parse()
}
