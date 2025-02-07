use http_snap::types::SnapResponse;
use http_snap::{client, comparer, parser};
use std::fs::{read_to_string, File};
use std::io::Write;

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
        let mut merges = Vec::new();
        for (i, parsed_response) in parsed_responses.iter().enumerate() {
            let merged = create_content_with_snapshot(request_texts[i], &parsed_response);
            merges.push(merged);
        }
        let merged = merges.join("\n\n###\n\n");

        let mut file = File::options()
            .write(true)
            .truncate(true)
            .open(path_to_file)?;
        file.write_all(&merged.as_bytes())
            .expect("Unable to write snapshot");
        file.flush()?;
    } else if execution_mode == Mode::CompareSnapshot {
        for (i, parsed_response) in parsed_responses.iter().enumerate() {
            let fits_snapshot = comparer::compare_to_snapshot(&http_files[i].snapshot, &parsed_response);
            if fits_snapshot {
                println!("Response {i} matches snapshot")
            } else {
                println!("Response {i} does NOT match snapshot");
            }
        }
    }

    return Ok(());
}

fn create_content_with_snapshot(raw_text: &str, response: &SnapResponse) -> String {
    let parts_of_file: Vec<&str> = raw_text.split("SNAPSHOT:").collect();
    let mut file_appending =
        "SNAPSHOT:\nstatus: ".to_owned() + &response.status.to_string() + "\n\n";
    for (name, value) in &response.headers {
        file_appending += &(name.as_str().to_owned() + ": " + value.to_str().unwrap());
        file_appending += "\n";
    }

    file_appending += "\n";
    file_appending += &serde_json::to_string_pretty(&response.body).unwrap();

    let merged = match parts_of_file.len() {
        1 => raw_text.to_owned() + "\n\n" + &file_appending,
        2 => parts_of_file[0].to_owned() + &file_appending,
        _ => panic!("Found more than one snapshot place"),
    };

    return merged;
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    UpdateSnapshot,
    CompareSnapshot,
}
