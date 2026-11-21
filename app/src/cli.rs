//! Command-line flags and the default data directory.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// forever-together: a peer-to-peer random dungeon finder for World of Warcraft Forever.
#[derive(Parser, Debug, Clone, PartialEq, Eq, Default)]
#[command(name = "forever-together", version, about)]
pub struct Cli {
    /// Run without a window or web server: only join the network and relay as a peer.
    #[arg(long)]
    pub headless: bool,
    /// Where to keep the identity, settings and known peers.
    #[arg(long, value_name = "DIR", global = true)]
    pub data_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Show, back up, restore or replace this install's key.
    #[command(subcommand)]
    Key(KeyCommand),
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum KeyCommand {
    /// Print this install's peer ID and public key. Creates a key if there is none.
    Show,
    /// Write the secret key as an unencrypted OpenSSH ed25519 key file (a new file, owner-only),
    /// or print it with --stdout. Creates a key if there is none.
    Export(ExportArgs),
    /// Replace the stored key with the one in an OpenSSH ed25519 key file.
    Import {
        /// The key file.
        file: PathBuf,
        /// Replace a key that is already stored.
        #[arg(long)]
        force: bool,
    },
    /// Discard the stored key and make a new one. Other nodes will see a new peer.
    Reset {
        /// Do not ask for confirmation.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct ExportArgs {
    /// Where to write the key. The file must not exist.
    #[arg(
        value_name = "FILE",
        conflicts_with = "stdout",
        required_unless_present = "stdout"
    )]
    pub file: Option<PathBuf>,
    /// Print the key to standard output instead of writing a file.
    #[arg(long)]
    pub stdout: bool,
}

/// What the process should do, decided from the flags alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Key(KeyCommand),
    Headless,
    Window,
}

impl Cli {
    /// A subcommand wins over `--headless`.
    pub fn mode(&self) -> Mode {
        if let Some(Command::Key(k)) = &self.command {
            Mode::Key(k.clone())
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
    fn the_old_print_public_key_flag_is_gone() {
        assert!(parse(&["--print-public-key"]).is_err());
    }

    #[test]
    fn key_subcommands_parse() {
        let mode = |a: &[&str]| parse(a).unwrap().mode();
        assert_eq!(mode(&["key", "show"]), Mode::Key(KeyCommand::Show));
        assert_eq!(
            mode(&["key", "export", "/tmp/k"]),
            Mode::Key(KeyCommand::Export(ExportArgs {
                file: Some(PathBuf::from("/tmp/k")),
                stdout: false
            }))
        );
        assert_eq!(
            mode(&["key", "export", "--stdout"]),
            Mode::Key(KeyCommand::Export(ExportArgs {
                file: None,
                stdout: true
            }))
        );
        assert_eq!(
            mode(&["key", "import", "k"]),
            Mode::Key(KeyCommand::Import {
                file: PathBuf::from("k"),
                force: false
            })
        );
        assert_eq!(
            mode(&["key", "import", "--force", "k"]),
            Mode::Key(KeyCommand::Import {
                file: PathBuf::from("k"),
                force: true
            })
        );
        assert_eq!(
            mode(&["key", "reset"]),
            Mode::Key(KeyCommand::Reset { yes: false })
        );
        assert_eq!(
            mode(&["key", "reset", "--yes"]),
            Mode::Key(KeyCommand::Reset { yes: true })
        );
    }

    #[test]
    fn export_needs_exactly_one_of_a_file_and_stdout() {
        assert!(parse(&["key", "export"]).is_err());
        assert!(parse(&["key", "export", "--stdout", "k"]).is_err());
    }

    #[test]
    fn key_commands_need_their_arguments_and_reject_unknowns() {
        assert!(parse(&["key"]).is_err());
        assert!(parse(&["key", "import"]).is_err());
        assert!(parse(&["key", "nope"]).is_err());
        assert!(parse(&["key", "reset", "--force"]).is_err());
        assert!(parse(&["key", "show", "extra"]).is_err());
    }

    #[test]
    fn data_dir_works_after_the_subcommand_and_a_subcommand_wins_over_headless() {
        let c = parse(&["key", "show", "--data-dir", "/tmp/d"]).unwrap();
        assert_eq!(c.resolve_data_dir(), PathBuf::from("/tmp/d"));
        assert_eq!(c.mode(), Mode::Key(KeyCommand::Show));
        let c = parse(&["--headless", "key", "show"]).unwrap();
        assert_eq!(c.mode(), Mode::Key(KeyCommand::Show));
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
