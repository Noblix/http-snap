use clap::{Args, Parser, Subcommand};
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
    },
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
                        Err(e) => eprintln!("Error expanding path: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Glob pattern error: {}", e),
        }
    } else {
        expanded.push(path);
    }
    return expanded;
}
