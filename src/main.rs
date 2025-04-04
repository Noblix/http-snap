use crate::cli::{expand_paths, Cli, Commands, GlobalOptions};
use clap::Parser;
use http_snap::parser::parse_environment;
use http_snap::types::{ExecuteOptions, Mode, Value};
use http_snap::{variable_generator};
use http_snap::{run, types};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let args = Cli::try_parse_from(vec![
        "http-snap",                  // program name (usually ignored)
        "test",
        "--path", "./http-examples/post-small-body.http",
        "--environment",
        "./http-examples/test_environment.txt",
    ])?;

    #[cfg(not(debug_assertions))]
    let args = Cli::parse();

    return match args.command {
        Commands::Test { global } => run_test(global).await,
        Commands::Update { global, options } => run_update(global, options).await,
    };
}

async fn run_test(global_options: GlobalOptions) -> Result<(), Box<dyn std::error::Error>> {
    setup_logging(global_options.verbose);
    let expanded_paths = expand_paths(global_options.path);
    let environment_variables = get_environment_variables(global_options.environment);
    let execute_options = ExecuteOptions::new_test();

    return execute(expanded_paths, environment_variables, execute_options).await;
}

async fn run_update(
    global_options: GlobalOptions,
    update_options: cli::UpdateOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    setup_logging(global_options.verbose);
    let expanded_paths = expand_paths(global_options.path);
    let environment_variables = get_environment_variables(global_options.environment);
    let execute_options = ExecuteOptions {
        mode: Mode::Update,
        update_options: Some(types::UpdateOptions {
            stop_on_failure: update_options.stop_on_failure,
            detectors: get_detectors(update_options.detectors),
        }),
    };

    return execute(expanded_paths, environment_variables, execute_options).await;
}

fn get_detectors(input: Vec<cli::Detector>) -> HashSet<types::Detector> {
    if input.contains(&cli::Detector::All) {
        return HashSet::from([types::Detector::Timestamp, types::Detector::Guid]);
    }
    // Otherwise, map each detector.
    input.iter().filter_map(|detector| {
        match detector {
            cli::Detector::Timestamp => Some(types::Detector::Timestamp),
            cli::Detector::Guid => Some(types::Detector::Guid),
            _ => None,
        }
    }).collect()
}

async fn execute(
    paths: Vec<PathBuf>,
    environment_variables: HashMap<String, Value>,
    execute_options: ExecuteOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut total_count = 0;
    let mut failed_count = 0;
    for path in paths {
        total_count += 1;
        log::info!("Running {:?}", path);
        let result = run(&path, &environment_variables, &execute_options).await;
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

fn setup_logging(verbose: bool) {
    let log_level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();
}

fn get_environment_variables(environment: Option<PathBuf>) -> HashMap<String, Value> {
    let mut env_variables = HashMap::new();
    if let Some(environment) = environment {
        let env_content = std::fs::read_to_string(environment).unwrap();
        let parsed_env_variables = parse_environment(&env_content).unwrap();
        env_variables = variable_generator::generate_variables(parsed_env_variables);
    }
    return env_variables;
}
