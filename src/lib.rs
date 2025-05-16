use crate::client::HttpResponse;
use crate::types::{
    ClientOptions, ExecuteOptions, ExecutedRequest, HttpFile, Mode, RawInput, UpdateOptions,
};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use std::{time};

pub mod client;
pub mod comparer;
pub mod detector;
pub mod merger;
pub mod parser;
pub mod request_extractor;
pub mod types;
pub mod variable_generator;
pub mod variable_store;

pub async fn run(
    path_to_file: &PathBuf,
    environment_variables: &HashMap<String, types::Value>,
    execute_options: &ExecuteOptions,
    client_options: &ClientOptions,
) -> Result<bool, Box<dyn std::error::Error>> {
    let extension = path_to_file.extension().unwrap();
    if extension == "http" {
        let (passed, file_content) = handle_http_file(
            path_to_file,
            environment_variables,
            execute_options,
            client_options,
        )
        .await?;
        if !passed && execute_options.mode == Mode::Update {
            let mut file = File::options()
                .write(true)
                .truncate(true)
                .open(path_to_file)?;

            file.write_all(&file_content.as_bytes())
                .expect("Unable to write snapshot");
            file.flush()?;
        }
        return Ok(passed);
    } else if extension == "md" {
        let (passed, sections_content) = handle_markdown_file(
            path_to_file,
            environment_variables,
            execute_options,
            client_options,
        )
        .await?;
        if !passed && execute_options.mode == Mode::Update {
            let raw_input = read_to_string(path_to_file).unwrap();
            let parser = Parser::new(&raw_input);

            let mut index = 0;
            let mut replaced = false;
            let mut in_http = false;
            let mapped = parser.map(|event| match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                    if lang.eq_ignore_ascii_case("http") =>
                {
                    in_http = true;
                    replaced = false;
                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_http = false;
                    event
                }
                Event::Text(_) if in_http => {
                    if !replaced {
                        let updated = sections_content[index].clone();
                        index += 1;
                        replaced = true;
                        Event::Text(updated.into())
                    } else {
                        Event::Text("".into())
                    }
                }
                other => other,
            });
            let mut content = String::new();
            cmark(mapped, &mut content).expect("Markdown re-emit failed");

            let mut file = File::options()
                .write(true)
                .truncate(true)
                .open(path_to_file)?;

            file.write_all(&content.as_bytes())
                .expect("Unable to write snapshot");
            file.flush()?;
        }
        return Ok(passed);
    } else {
        panic!("Unknown file format")
    }
}

async fn handle_markdown_file(
    path_to_file: &PathBuf,
    environment_variables: &HashMap<String, types::Value>,
    execute_options: &ExecuteOptions,
    client_options: &ClientOptions,
) -> Result<(bool, Vec<String>), Box<dyn std::error::Error>> {
    let requests = request_extractor::extract_requests(path_to_file);
    let stop_on_failure = get_stop_on_failure_option(&execute_options);
    let (passed, raw_snapshots) = run_requests(
        requests,
        environment_variables,
        stop_on_failure,
        client_options,
    )
    .await?;
    let final_snapshots = detect_patterns(raw_snapshots, &execute_options.update_options);

    let mut recombined_sections = Vec::new();
    let mut section_index = 0;
    let mut previous_import = None;
    let mut section = Vec::new();
    for snapshot in final_snapshots {
        if snapshot.raw_input.imported_path != None {
            if previous_import != snapshot.raw_input.imported_path {
                previous_import = snapshot.raw_input.imported_path.clone();
                section.push(snapshot);
            }
            continue;
        }

        if snapshot.raw_input.section != section_index {
            if !section.is_empty() {
                recombined_sections.push(section);
            }

            section_index = snapshot.raw_input.section.clone();
            section = Vec::new();
        }
        section.push(snapshot);
    }
    recombined_sections.push(section);

    let mut sections_content = Vec::new();
    for section in recombined_sections {
        sections_content.push(create_http_content(
            &section,
            &execute_options.update_options,
        ))
    }
    return Ok((passed, sections_content));
}

async fn handle_http_file(
    path_to_file: &PathBuf,
    environment_variables: &HashMap<String, types::Value>,
    execute_options: &ExecuteOptions,
    client_options: &ClientOptions,
) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let requests = request_extractor::extract_requests(path_to_file);
    let stop_on_failure = get_stop_on_failure_option(&execute_options);
    let (passed, raw_snapshots) = run_requests(
        requests,
        environment_variables,
        stop_on_failure,
        client_options,
    )
    .await?;
    let final_snapshots = detect_patterns(raw_snapshots, &execute_options.update_options);
    let file_content = create_http_content(&final_snapshots, &execute_options.update_options);
    return Ok((passed, file_content));
}

async fn run_requests(
    inputs: Vec<RawInput>,
    environment_variables: &HashMap<String, types::Value>,
    stop_on_failure: bool,
    client_options: &ClientOptions,
) -> Result<(bool, Vec<ExecutedRequest>), Box<dyn std::error::Error>> {
    let mut passed = true;
    let mut executed_requests = Vec::new();
    for input in &inputs {
        executed_requests.push(ExecutedRequest {
            raw_input: input.clone(),
            snapshot: None,
        });
    }

    let mut variable_store = variable_store::VariableStore::new();
    variable_store.extend_variables(&environment_variables);

    let client = client::HttpClient::new(client_options);
    for (index, request) in inputs.into_iter().enumerate() {
        let delay_section = parser::try_parse_delay(&request.text).unwrap();
        if let Some(delay) = delay_section {
            tokio::time::sleep(time::Duration::from_millis(delay)).await;
            continue;
        }

        let http_file = parser::parse_file(&request.text).unwrap();
        let http_file_without_variables = variable_store.replace_variables(http_file);
        log_variable_store(&variable_store);

        log_request(&http_file_without_variables);
        let response = client.send_request(&http_file_without_variables).await?;
        log_response(&response);

        let parsed_response = parser::parse_response(&response).await?;

        let mut matched_option = false;
        for (option_index, snapshot) in http_file_without_variables.snapshots.iter().enumerate() {
            matched_option = comparer::compare_to_snapshot(&snapshot, &parsed_response);
            if matched_option {
                log_option_match(index, option_index);
                variable_store.update_variables(&snapshot, &parsed_response);
                break;
            }
        }

        if !matched_option {
            passed = false;
            log::error!("Snapshot {0} did NOT match", index + 1);
            executed_requests[index].snapshot = Some(parsed_response);
            if stop_on_failure {
                break;
            }
        }
    }

    return Ok((passed, executed_requests));
}

fn get_stop_on_failure_option(execute_options: &&ExecuteOptions) -> bool {
    if let Some(update_options) = &execute_options.update_options {
        update_options.stop_on_failure
    } else {
        true
    }
}

fn detect_patterns(
    executed_requests: Vec<ExecutedRequest>,
    update_options: &Option<UpdateOptions>,
) -> Vec<ExecutedRequest> {
    let replacer = detector::Replacer::new(update_options);
    let mut final_executed_requests = Vec::new();

    for executed_request in executed_requests {
        if let Some(snapshot) = executed_request.snapshot {
            let new_snapshot = replacer.detect_types(snapshot);
            final_executed_requests.push(ExecutedRequest {
                raw_input: executed_request.raw_input,
                snapshot: Some(new_snapshot),
            });
        } else {
            final_executed_requests.push(executed_request);
        }
    }
    return final_executed_requests;
}

fn create_http_content(
    executed_requests: &Vec<ExecutedRequest>,
    update_options: &Option<UpdateOptions>,
) -> String {
    let mut imports = Vec::new();
    let mut merged = Vec::new();
    for executed_request in executed_requests {
        if let Some(import_path) = &executed_request.raw_input.imported_path {
            imports.push(format!("import {}", import_path.display()));
            continue;
        }
        if let Some(snapshot) = &executed_request.snapshot {
            if let Some(options) = update_options {
                let snapshot_as_str = merger::create_content_with_snapshot(
                    &executed_request.raw_input.text,
                    &snapshot,
                    &options.update_mode,
                );
                merged.push(snapshot_as_str);
            }
        } else {
            merged.push(executed_request.raw_input.text.clone());
        }
    }

    let mut result = String::new();
    if !imports.is_empty() {
        result.push_str(&imports.join("\n"));
        result.push_str("\n\n");
    }

    result.push_str(&merged.join("\n\n###\n\n"));

    return result;
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

fn log_option_match(index: usize, option_index: usize) {
    log::debug!(
        "Snapshot {0} matches on option {1}",
        index + 1,
        option_index + 1
    );
}
