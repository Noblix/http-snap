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
) -> Result<(), Box<dyn std::error::Error>> {
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let request_texts: Vec<&str> = text.split("###").map(|s| s.trim()).collect();

    let client = client::HttpClient::new();
    let mut variable_store = variable_store::VariableStore::new();
    for (index, request_text) in request_texts.clone().iter().enumerate() {
        let http_file = parser::parse_file(request_text).unwrap();
        let http_file_without_variables = variable_store.replace_variables(http_file);

        let response = client.send_request(&http_file_without_variables).await?;
        let parsed_response =
            parser::parse_response(http_file_without_variables.options, &response).await?;

        let are_equal =
            comparer::compare_to_snapshot(&http_file_without_variables.snapshot, &parsed_response);

        if are_equal {
            println!("Snapshot {index} matches!");
            variable_store.update_variables(&http_file_without_variables.snapshot, &parsed_response);
        } else if should_update {
            merger::merge_snapshots_into_files(
                path_to_file,
                &request_texts,
                index,
                parsed_response,
            )?;
            break;
        } else {
            break;
        }
    }

    return Ok(());
}
