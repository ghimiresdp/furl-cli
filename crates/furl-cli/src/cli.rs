use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None, arg_required_else_help(true))]
pub struct FurlCliArgs {
    /// url to download the file from
    #[arg()]
    pub url: String,

    /// output directory, defaults to the current directory
    #[arg(short, long, default_value_t = String::from("."))]
    pub out: String,

    /// output filename, defaults to the filename in the url
    #[arg(short, long)]
    pub filename: Option<String>,

    /// Number of threads, maximum allowed 255
    #[arg(short, long, default_value_t = 8, value_parser = clap::value_parser!(u8).range(1..=255))]
    pub threads: u8,

    /// Number of chunks in MB, maximum allowed 100
    #[arg(short, long, default_value_t = 10, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub chunksize: u8,
}
