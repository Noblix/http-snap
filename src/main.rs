use crate::cli::{expand_paths, Cli, Commands};
use clap::Parser;
use http_snap::run;

mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let (path, use_test_mode) = match args.command {
        Commands::Test { global } => (global.path, true),
        Commands::Update { global } => (global.path, false),
    };

    let expanded_paths = expand_paths(path);
    for path in expanded_paths {
        println!("{:?}", path);
        let _result = run(&path, use_test_mode).await;
    }
    return Ok(());
}
