use clap::{Args, Parser, Subcommand};
use furl_core::BasicAuth;

use crate::config::parse_size;

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
    /// view or manage furl's configuration.
    ///
    /// `furl config` lists every value, `furl config <key>` prints one
    /// value, `furl config <key> <value>` sets and saves it, and
    /// `furl config --reset` restores the defaults.
    Config {
        /// configuration key, e.g. threads, max_chunk_size, download_dir
        #[arg(conflicts_with = "reset")]
        key: Option<String>,
        /// value to set `key` to; omit to just read the current value.
        /// max_chunk_size accepts a human-readable size (e.g. 512B, 10KB,
        /// 5MB, 1GB) or a plain byte count
        #[arg(conflicts_with = "reset")]
        value: Option<String>,
        /// reset the configuration file back to its defaults
        #[arg(long)]
        reset: bool,
    },
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

    /// Maximum size of a download chunk, in bytes; also accepts a unit
    /// suffix, e.g. 512KB, 5MB, 1GB. Defaults to the value from the config
    /// file, or 10MB if not configured
    #[arg(short, long, value_parser = parse_size)]
    pub chunksize: Option<u64>,

    /// Credentials for HTTP Basic Authentication, as USER[:PASSWORD]. If the
    /// password is omitted, you will be prompted for it, which keeps it out
    /// of your shell history
    #[arg(short, long, value_name = "USER[:PASSWORD]", value_parser = parse_user)]
    pub user: Option<BasicAuth>,
}

/// Splits `USER[:PASSWORD]` at the first `:`, the same way curl's `--user`
/// does, so passwords may themselves contain colons.
fn parse_user(input: &str) -> Result<BasicAuth, String> {
    let (username, password) = match input.split_once(':') {
        Some((username, password)) => (username, Some(password.to_string())),
        None => (input, None),
    };
    if username.is_empty() {
        return Err("username must not be empty".to_string());
    }
    Ok(BasicAuth {
        username: username.to_string(),
        password,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_url_as_download_without_subcommand() {
        let args = FurlCliArgs::try_parse_from(["furl", "https://example.com/file.zip"]).unwrap();

        assert!(args.command.is_none());
        assert_eq!(
            args.download.url,
            Some("https://example.com/file.zip".to_string())
        );
    }

    #[test]
    fn parses_download_options() {
        let args = FurlCliArgs::try_parse_from([
            "furl",
            "https://example.com/file.zip",
            "--out",
            "/tmp",
            "--filename",
            "file.zip",
            "--threads",
            "16",
            "--chunksize",
            "20",
        ])
        .unwrap();

        assert_eq!(args.download.out, Some("/tmp".to_string()));
        assert_eq!(args.download.filename, Some("file.zip".to_string()));
        assert_eq!(args.download.threads, Some(16));
        assert_eq!(args.download.chunksize, Some(20));
    }

    #[test]
    fn parses_chunksize_with_explicit_unit() {
        let args = FurlCliArgs::try_parse_from([
            "furl",
            "https://example.com/file.zip",
            "--chunksize",
            "512KB",
        ])
        .unwrap();

        assert_eq!(args.download.chunksize, Some(512 * 1024));
    }

    #[test]
    fn rejects_chunksize_with_unknown_unit() {
        let result = FurlCliArgs::try_parse_from([
            "furl",
            "https://example.com/file.zip",
            "--chunksize",
            "5XB",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_threads_above_range() {
        let result = FurlCliArgs::try_parse_from([
            "furl",
            "https://example.com/file.zip",
            "--threads",
            "256",
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_user_with_password() {
        let args = FurlCliArgs::try_parse_from([
            "furl",
            "https://example.com/file.zip",
            "--user",
            "alice:s3cr:et",
        ])
        .unwrap();

        let auth = args.download.user.unwrap();
        assert_eq!(auth.username, "alice");
        // only the first colon separates username from password
        assert_eq!(auth.password.as_deref(), Some("s3cr:et"));
    }

    #[test]
    fn parses_user_without_password() {
        let args =
            FurlCliArgs::try_parse_from(["furl", "https://example.com/file.zip", "-u", "alice"])
                .unwrap();

        let auth = args.download.user.unwrap();
        assert_eq!(auth.username, "alice");
        assert_eq!(auth.password, None);
    }

    #[test]
    fn parses_user_with_empty_password() {
        let args =
            FurlCliArgs::try_parse_from(["furl", "https://example.com/file.zip", "-u", "alice:"])
                .unwrap();

        assert_eq!(args.download.user.unwrap().password.as_deref(), Some(""));
    }

    #[test]
    fn rejects_user_with_empty_username() {
        let result =
            FurlCliArgs::try_parse_from(["furl", "https://example.com/file.zip", "-u", ":secret"]);

        assert!(result.is_err());
    }

    #[test]
    fn parses_bare_config_as_list() {
        let args = FurlCliArgs::try_parse_from(["furl", "config"]).unwrap();

        assert!(matches!(
            args.command,
            Some(FurlCommand::Config {
                key: None,
                value: None,
                reset: false
            })
        ));
    }

    #[test]
    fn parses_config_with_key_as_get() {
        let args = FurlCliArgs::try_parse_from(["furl", "config", "threads"]).unwrap();

        assert!(matches!(
            args.command,
            Some(FurlCommand::Config {
                key: Some(ref key),
                value: None,
                reset: false
            }) if key == "threads"
        ));
    }

    #[test]
    fn parses_config_with_key_and_value_as_set() {
        let args = FurlCliArgs::try_parse_from(["furl", "config", "threads", "16"]).unwrap();

        assert!(matches!(
            args.command,
            Some(FurlCommand::Config {
                key: Some(ref key),
                value: Some(ref value),
                reset: false
            }) if key == "threads" && value == "16"
        ));
    }

    #[test]
    fn parses_config_reset_flag() {
        let args = FurlCliArgs::try_parse_from(["furl", "config", "--reset"]).unwrap();

        assert!(matches!(
            args.command,
            Some(FurlCommand::Config {
                key: None,
                value: None,
                reset: true
            })
        ));
    }

    #[test]
    fn rejects_reset_combined_with_key() {
        let result = FurlCliArgs::try_parse_from(["furl", "config", "threads", "--reset"]);

        assert!(result.is_err());
    }
}
