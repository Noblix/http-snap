use crate::types::SnapResponse;
use std::fs::read_to_string;

pub mod client;
pub mod comparer;
pub mod merger;
pub mod parser;
pub mod snapshot_types;
pub mod types;
pub mod variable_store;

pub async fn run(
    path_to_file: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let request_texts: Vec<&str> = text.split("###").map(|s| s.trim()).collect();

    let client = client::HttpClient::new();
    let mut variable_store = variable_store::VariableStore::new();
    let mut previous: Option<SnapResponse> = None;
    for (index, request_text) in request_texts.clone().iter().enumerate() {
        let text_without_variables = variable_store.replace_variables(request_text, &previous);
        let http_file = parser::parse_file(&text_without_variables).unwrap();

        let response = client.send_request(&http_file).await?;
        let parsed_response = parser::parse_response(http_file.options, &response).await?;

        let are_equal = comparer::compare_to_snapshot(&http_file.snapshot, &parsed_response);

        if are_equal {
            previous = Some(parsed_response);
            println!("Snapshot {index} matches!")
        } else {
            merger::merge_snapshots_into_files(path_to_file, &request_texts, index, parsed_response)?;
            break;
        }
    }

    return Ok(());
}
