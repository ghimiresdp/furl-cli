use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None, arg_required_else_help(true))]
pub struct FurlCliArgs {
    #[command(subcommand)]
    pub command: Option<FurlCommand>,

    #[command(flatten)]
    pub download: DownloadArgs,
}

#[derive(Debug, Subcommand)]
pub enum FurlCommand {
    /// view or manage furl's configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// print the effective configuration (config file values, defaults for
    /// anything not set)
    List,
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// url to download the file from
    #[arg()]
    pub url: Option<String>,

    /// output directory, defaults to the value from the config file, or the
    /// current directory if not configured
    #[arg(short, long)]
    pub out: Option<String>,

    /// output filename, defaults to the filename in the url
    #[arg(short, long)]
    pub filename: Option<String>,

    /// Number of threads, maximum allowed 255. Defaults to the value from
    /// the config file, or 4 if not configured
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub threads: Option<u8>,

    /// Number of chunks in MB, maximum allowed 100. Defaults to the value
    /// from the config file, or 10 if not configured
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub chunksize: Option<u8>,
}
