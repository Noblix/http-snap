use crate::client::HttpResponse;
use crate::types::{ExecuteOptions, HttpFile, Mode, RawInput, UpdateMode, UpdateOptions};
use itertools::Itertools;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

pub mod client;
pub mod comparer;
pub mod detector;
pub mod merger;
pub mod parser;
pub mod types;
pub mod variable_generator;
pub mod variable_store;

pub async fn run(
    path_to_file: &PathBuf,
    environment_variables: &HashMap<String, types::Value>,
    execute_options: &ExecuteOptions,
) -> Result<bool, Box<dyn std::error::Error>> {
    let request_texts = extract_requests(path_to_file);

    let mut variable_store = variable_store::VariableStore::new();
    variable_store.extend_variables(&environment_variables);

    let client = client::HttpClient::new();
    let replacer = detector::Replacer::new(&execute_options.update_options);
    let mut passed = true;
    let mut updated_snapshots = Vec::new();
    for (index, request) in request_texts.clone().iter().enumerate() {
        let http_file = parser::parse_file(&request.text).unwrap();
        let http_file_without_variables = variable_store.replace_variables(http_file);
        log_variable_store(&variable_store);

        log_request(&http_file_without_variables);
        let response = client.send_request(&http_file_without_variables).await?;
        log_response(&response);

        let parsed_response =
            parser::parse_response(http_file_without_variables.options, &response).await?;

        let mut matched_option = false;
        for (option_index, snapshot) in http_file_without_variables.snapshots.iter().enumerate() {
            matched_option = comparer::compare_to_snapshot(&snapshot, &parsed_response);
            if matched_option {
                log::debug!(
                    "Snapshot {0} matches on option {1}",
                    index + 1,
                    option_index + 1
                );
                variable_store.update_variables(&snapshot, &parsed_response);
                break;
            }
        }

        if !matched_option {
            passed = false;

            let new_snapshot = replacer.detect_types(parsed_response);
            updated_snapshots.push((index, new_snapshot));

            log::error!("Snapshot {0} did NOT match", index + 1);

            if matches!(
                &execute_options.update_options,
                Some(UpdateOptions {
                    stop_on_failure: true,
                    ..
                })
            ) {
                break;
            }
        }
    }

    // TODO: Add && !passed
    if &execute_options.mode == &Mode::Update {
        let update_mode = if let Some(mode) = &execute_options.update_options {
            &mode.update_mode
        } else {
            &UpdateMode::Overwrite
        };
        merger::merge_snapshots_into_files(
            path_to_file,
            &request_texts,
            updated_snapshots,
            &update_mode,
        )?;
    }

    return Ok(passed);
}

fn extract_requests(path_to_file: &PathBuf) -> Vec<RawInput> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let mut request_texts = Vec::new();

    let (files_to_import, text_without_imports) = extract_imports(text);
    for file in files_to_import {
        let base_dir = path_to_file.parent().unwrap_or_else(|| Path::new(""));
        let full_path = base_dir.join(&file);

        let imported_requests = extract_requests(&full_path);

        for request in imported_requests {
            request_texts.push(RawInput {
                text: request.text,
                imported_path: Some(PathBuf::from(&file)),
            })
        }
    }

    for request in text_without_imports.split("###") {
        request_texts.push(RawInput {
            text: request.trim().to_string(),
            imported_path: None,
        })
    }
    return request_texts;
}

fn extract_imports(text: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut index = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("import ") {
            imports.push(path.to_string());
        } else if !trimmed.is_empty() {
            // once we hit a non-import, non-blank line, stop
            break;
        }
        index += 1;
    }
    return (imports, text.lines().skip(index).join("\n"));
}

fn log_variable_store(variable_store: &variable_store::VariableStore) {
    if variable_store.variables.is_empty() {
        log::debug!("Variable store is empty");
    } else {
        let variables = variable_store
            .variables
            .iter()
            .map(|(key, value)| format!("{key}: {}", serde_json::to_string_pretty(value).unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        log::debug!("Variable store contains: \n{}", variables);
    }
}

fn log_request(request: &HttpFile) {
    let headers = request
        .headers
        .iter()
        .map(|header| format!("{}: {}", header.name, header.value))
        .collect::<Vec<_>>()
        .join("\n");
    let body_pretty = serde_json::to_string_pretty(&request.body).unwrap();

    let log_message = format!(
        "Sending {:?} {}\nHeaders:\n{}\nBody:\n{}",
        request.verb, request.url, headers, body_pretty
    );
    log::debug!("{}", log_message);
}

fn log_response(response: &HttpResponse) {
    let headers = response
        .headers
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value.to_str().unwrap()))
        .collect::<Vec<_>>()
        .join("\n");

    let body_pretty = match serde_json::from_str::<Value>(&response.body) {
        Ok(json) => &serde_json::to_string_pretty(&json).unwrap(),
        Err(_) => &response.body,
    };

    let log_message = format!(
        "Response:\nStatus: {}\nHeaders:\n{}\nBody:\n{}",
        response.status, headers, body_pretty
    );

    log::debug!("{}", log_message);
}
