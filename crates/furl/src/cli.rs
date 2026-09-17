use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None, arg_required_else_help(true))]
pub struct FurlCliArgs {
    /// url to download the file from
    #[arg()]
    pub url: String,

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
