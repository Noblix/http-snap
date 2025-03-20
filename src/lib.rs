use crate::client::HttpResponse;
use crate::types::HttpFile;
use serde_json::Value;
use std::fs::read_to_string;
use std::path::PathBuf;

pub mod client;
pub mod comparer;
pub mod merger;
pub mod parser;
pub mod types;
pub mod variable_store;

pub async fn run(
    path_to_file: &PathBuf,
    should_update: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let request_texts: Vec<&str> = text.split("###").map(|s| s.trim()).collect();

    let client = client::HttpClient::new();
    let mut variable_store = variable_store::VariableStore::new();
    for (index, request_text) in request_texts.clone().iter().enumerate() {
        let http_file = parser::parse_file(request_text).unwrap();
        let http_file_without_variables = variable_store.replace_variables(http_file);
        log_variable_store(&variable_store);

        log_request(&http_file_without_variables);
        let response = client.send_request(&http_file_without_variables).await?;
        log_response(&response);

        let parsed_response =
            parser::parse_response(http_file_without_variables.options, &response).await?;

        let are_equal =
            comparer::compare_to_snapshot(&http_file_without_variables.snapshot, &parsed_response);

        if are_equal {
            log::debug!("Snapshot {0} matches!", index + 1);
            variable_store
                .update_variables(&http_file_without_variables.snapshot, &parsed_response);
        } else {
            if should_update {
                merger::merge_snapshots_into_files(
                    path_to_file,
                    &request_texts,
                    index,
                    parsed_response,
                )?;
            }

            log::error!("Snapshot {0} did NOT match", index + 1);
            return Ok(false);
        }
    }

    return Ok(true);
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
