use http_snap::{client, comparer, merger, parser};
use std::fs::{read_to_string};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Options that should come from CLI
    let path_to_file = "http-examples/todo-app/cannot_create_todo_item_with_no_text.http";
    let execution_mode = Mode::CompareSnapshot;

    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let request_texts: Vec<&str> = text.split("###").map(|s| s.trim()).collect();

    let client = client::HttpClient::new();
    let mut http_files = Vec::new();
    let mut parsed_responses = Vec::new();
    for request_text in request_texts.clone() {
        let http_file = parser::parse_file(request_text).unwrap();

        let response = client.send_request(&http_file).await?;
        let parsed_response = parser::parse_response(&response).await?;

        http_files.push(http_file);
        parsed_responses.push(parsed_response);
    }

    if execution_mode == Mode::UpdateSnapshot {
        merger::merge_snapshots_into_files(parsed_responses, request_texts, path_to_file)?;
    } else if execution_mode == Mode::CompareSnapshot {
        comparer::compare_to_snapshots(&http_files, &parsed_responses);
    }

    return Ok(());
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    UpdateSnapshot,
    CompareSnapshot,
}
