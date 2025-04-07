use clap::{Args, Parser, Subcommand, ValueEnum};
use glob::glob;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    about = "HttpSnap is used to perform snapshot tests at an http level",
    version = "1.0"
)]
pub(crate) struct Cli {
    /// Subcommand to run (`test` or `update`)
    #[command(subcommand)]
    pub(crate) command: Commands,
}

/// Global options to be reused in each subcommand
#[derive(Debug, Args)]
pub(crate)struct GlobalOptions {
    /// File path or directory to process (supports wildcards)
    #[arg(long, required = true)]
    pub(crate)path: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    pub(crate) verbose: bool,

    /// File containing environment variables
    #[arg(short, long)]
    pub(crate) environment: Option<PathBuf>,
}

/// Enum of subcommands (test and update)
#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Runs all tests and compares responses to snapshots
    Test {
        #[command(flatten)]
        global: GlobalOptions,
    },

    /// Updates snapshots on mismatches, with selectable mode
    Update {
        #[command(flatten)]
        global: GlobalOptions,

        #[command(flatten)]
        options: UpdateOptions,
    },
}

#[derive(Debug, Args)]
pub struct UpdateOptions {
    /// Continue updating tests in spite of response mismatches
    #[arg(long = "continue-on-failure", action = clap::ArgAction::SetFalse, default_value_t = true)]
    pub(crate) stop_on_failure: bool,

    /// Choose how mismatching snapshots should be updated.
    #[arg(long, value_enum, default_value_t = UpdateMode::Overwrite)]
    pub(crate) update_mode: UpdateMode,

    /// Choose which detectors to run. Can be specified multiple times.
    #[arg(long, value_enum, value_delimiter = ',', num_args = 1..)]
    pub(crate) detectors: Vec<Detector>
}

#[derive(Debug, ValueEnum, Clone, PartialEq, Eq)]
pub enum UpdateMode {
    Overwrite,
    Append,
}

#[derive(Debug, ValueEnum, Clone, PartialEq, Eq)]
pub enum Detector {
    All,
    Timestamp,
    Guid,
}

pub(crate) fn expand_paths(path: PathBuf) -> Vec<PathBuf> {
    let mut expanded = Vec::new();
    let path_str = path.to_string_lossy();
    // Check if the path contains a wildcard
    if path_str.contains('*') {
        // Use the glob crate to find matching files
        match glob(&path_str) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(p) => expanded.push(p),
                        Err(e) => log::error!("Error expanding path: {}", e),
                    }
                }
            }
            Err(e) => log::error!("Glob pattern error: {}", e),
        }
    } else {
        expanded.push(path);
    }
    return expanded;
}
