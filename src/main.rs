use crate::cli::{expand_paths, Cli, Commands};
use clap::Parser;
use http_snap::run;

mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let (options, should_update, stop_on_failure) = match args.command {
        Commands::Test { global } => (global, false, true),
        Commands::Update { global, options } => (global, true, options.stop_in_failure),
    };

    let log_level = if options.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    let expanded_paths = expand_paths(options.path);
    let mut total_count = 0;
    let mut failed_count = 0;
    for path in expanded_paths {
        total_count += 1;
        log::info!("Running {:?}", path);
        let result = run(&path, should_update, stop_on_failure).await;
        if result? {
            log::info!("Test {:?} passed", path);
        } else {
            failed_count += 1;
            log::error!("Test {:?} failed", path);
        }
    }

    log::info!(
        "Ran {total_count} tests: {0} passed and {failed_count} failed",
        total_count - failed_count
    );

    return Ok(());
}
