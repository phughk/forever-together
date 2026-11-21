//! Command-line flags and the default data directory.

use clap::Parser;
use std::path::PathBuf;

/// forever-together: a peer-to-peer random dungeon finder for World of Warcraft Forever.
#[derive(Parser, Debug, Clone, PartialEq, Eq, Default)]
#[command(name = "forever-together", version, about)]
pub struct Cli {
    /// Run without a window or web server: only join the network and relay as a peer.
    #[arg(long)]
    pub headless: bool,
    /// Print this install's public key (the peer ID) and exit.
    #[arg(long)]
    pub print_public_key: bool,
    /// Where to keep the identity, settings and known peers.
    #[arg(long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,
}

/// What the process should do, decided from the flags alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    PrintPublicKey,
    Headless,
    Window,
}

impl Cli {
    /// `--print-public-key` wins over `--headless`.
    pub fn mode(&self) -> Mode {
        if self.print_public_key {
            Mode::PrintPublicKey
        } else if self.headless {
            Mode::Headless
        } else {
            Mode::Window
        }
    }

    /// The flag, or the platform's per-user data directory.
    pub fn resolve_data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| {
            default_data_dir(
                |k| std::env::var_os(k).map(PathBuf::from),
                std::env::consts::OS,
            )
        })
    }
}

const APP_DIR: &str = "forever-together";

/// The per-user data directory for `os`, reading the environment through `env`.
pub fn default_data_dir(env: impl Fn(&str) -> Option<PathBuf>, os: &str) -> PathBuf {
    let home = || env("HOME").unwrap_or_else(|| PathBuf::from("."));
    match os {
        "macos" => home().join("Library/Application Support").join(APP_DIR),
        "windows" => env("APPDATA")
            .unwrap_or_else(|| home().join("AppData/Roaming"))
            .join(APP_DIR),
        _ => env("XDG_DATA_HOME")
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home().join(".local/share"))
            .join(APP_DIR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("forever-together").chain(args.iter().copied()))
    }

    #[test]
    fn no_flags_means_a_window() {
        let c = parse(&[]).unwrap();
        assert_eq!(c, Cli::default());
        assert_eq!(c.mode(), Mode::Window);
    }

    #[test]
    fn headless_and_data_dir() {
        let c = parse(&["--headless", "--data-dir", "/tmp/x"]).unwrap();
        assert_eq!(c.mode(), Mode::Headless);
        assert_eq!(c.resolve_data_dir(), PathBuf::from("/tmp/x"));
        let c = parse(&["--data-dir=rel/dir"]).unwrap();
        assert_eq!(c.data_dir, Some(PathBuf::from("rel/dir")));
        assert_eq!(c.mode(), Mode::Window);
    }

    #[test]
    fn print_public_key_wins() {
        let c = parse(&["--print-public-key", "--headless"]).unwrap();
        assert_eq!(c.mode(), Mode::PrintPublicKey);
    }

    #[test]
    fn unknown_flags_and_a_missing_value_are_errors() {
        assert!(parse(&["--nope"]).is_err());
        assert!(parse(&["--data-dir"]).is_err());
        assert!(parse(&["--headless=maybe"]).is_err());
    }

    #[test]
    fn default_data_dirs() {
        let env = |k: &str| match k {
            "HOME" => Some(PathBuf::from("/home/u")),
            "XDG_DATA_HOME" => Some(PathBuf::from("/xdg")),
            "APPDATA" => Some(PathBuf::from("C:/Roaming")),
            _ => None,
        };
        assert_eq!(
            default_data_dir(env, "macos"),
            PathBuf::from("/home/u/Library/Application Support/forever-together")
        );
        assert_eq!(
            default_data_dir(env, "linux"),
            PathBuf::from("/xdg/forever-together")
        );
        assert_eq!(
            default_data_dir(env, "windows"),
            PathBuf::from("C:/Roaming/forever-together")
        );
        let bare = |k: &str| (k == "HOME").then(|| PathBuf::from("/home/u"));
        assert_eq!(
            default_data_dir(bare, "linux"),
            PathBuf::from("/home/u/.local/share/forever-together")
        );
    }
}
