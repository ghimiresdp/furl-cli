use std::process::exit;

use furl_core::DownloadConfig;

use super::{format_config, get, load_config, persist, set};

/// Handles `furl config [key] [value] [--reset]`:
/// - no key: prints the full effective configuration
/// - key only: prints that value
/// - key and value: sets and saves it
/// - `--reset`: restores the defaults (the CLI rejects combining this with
///   `key`/`value`, so it always wins here)
pub fn handle(key: Option<String>, value: Option<String>, reset: bool) {
    if reset {
        let defaults = DownloadConfig::default();
        if let Err(err) = persist(&defaults) {
            eprintln!("Error: {err}");
            exit(1);
        }
        print!("{}", format_config(&defaults));
        return;
    }

    let config = load_config();

    let Some(key) = key else {
        print!("{}", format_config(&config));
        return;
    };

    let Some(value) = value else {
        match get(&config, &key) {
            Some(value) => println!("{value}"),
            None => {
                eprintln!("Error: unknown configuration key '{key}'");
                exit(1);
            }
        }
        return;
    };

    match set(config, &key, &value) {
        Ok(updated) => {
            if let Err(err) = persist(&updated) {
                eprintln!("Error: {err}");
                exit(1);
            }
            println!("{key} = {}", get(&updated, &key).unwrap());
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(1);
        }
    }
}
