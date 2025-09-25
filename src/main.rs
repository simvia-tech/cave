//! Entry point for the `cave` CLI application.
//!
//! This binary orchestrates user commands by parsing CLI arguments
//! and dispatching them to the corresponding module functions.
//! Errors are handled per-command and printed to `stderr` before exiting
//! with a non-zero status when necessary.
//! 
//! The structure of the cli is described in the cli.rs file. It's in this file you can 
//! modify the cli's commands.

mod cli;
mod manage;
mod docker;
mod config;
mod telemetry;

use std::io;
use clap::Parser;
use cli::{Cli, Command, ConfigAction};
use manage::*;
use config::*;
use std::process;

/// Entry point for the `cave` CLI binary.
///
/// This function:
/// 1. Parses the CLI arguments and subcommands using [Clap](https://docs.rs/clap).
/// 2. Loads the user configuration.
/// 3. Matches the chosen subcommand and dispatches it to the relevant handler.
/// 4. Prints errors to `stderr` and exits with code `1` if a command fails.
///
/// # Errors
/// Returns any [`io::Error`] if CLI parsing, config reading, or underlying commands fail.
/// Errors from subcommands are printed and cause an exit with code `1`.
fn main() -> io::Result<()> {
    let args = Cli::parse();
    let _ = match read_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let result = match args.command {
        Command::Use { version } => set_version(version, true),
        Command::Pin { version } => set_version(version, false),
        Command::Run { args } => run_aster(&args),
        Command::List { prefix } => print_local_versions(prefix.unwrap_or_default()),
        Command::Available { prefix } => print_remote_versions(prefix.unwrap_or_default()),
        Command::Config { action } => {
            match action {
                ConfigAction::EnableAutoUpdate => set_auto_update(true),
                ConfigAction::DisableAutoUpdate => set_auto_update(false),
                ConfigAction::EnableUsageTracking => set_version_tracking(true),
                ConfigAction::DisableUsageTracking => set_version_tracking(false),
                // TODO : uncomment to have registry option
                //
                // ConfigAction::SetRegistry { repo, user, token } => {
                //     set_registry(Some(Registry { repo, user, token }))
                // }
                // ConfigAction::EraseRegistry => set_registry(None),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }

    Ok(())
}
