use http_snap::{client, parser};
use std::fs::{read_to_string, File};
use std::io::Write;
use http_snap::types::SnapResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_to_file = "http-examples/post-simple.http";
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let http_file = parser::parse_file(text).unwrap();
    println!("{:?}", http_file);

    let response = client::send_request(&http_file).await?;
    let parsed_response = parser::parse_response(response).await?;

    let merged = create_content_with_snapshot(&raw_text, &parsed_response);

    let mut file = File::options()
        .write(true)
        .truncate(true)
        .open(path_to_file)?;
    file.write_all(&merged.as_bytes())
        .expect("Unable to write snapshot");
    file.flush()?;

    return Ok(());
}

fn create_content_with_snapshot(raw_text: &str, response: &SnapResponse) -> String {
    let parts_of_file: Vec<&str> = raw_text.split("SNAPSHOT:").collect();
    let mut file_appending = "SNAPSHOT:\nstatus: ".to_owned() + &response.status.to_string() + "\n\n";
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