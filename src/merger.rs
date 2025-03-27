use crate::types::SnapResponse;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn merge_snapshots_into_files(
    path_to_file: &PathBuf,
    request_texts: &Vec<&str>,
    index: usize,
    parsed_response: SnapResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut merges = Vec::new();
    for (i, raw_text) in request_texts.iter().enumerate() {
        let mut merged = raw_text.to_string();
        if i == index {
            merged = create_content_with_snapshot(raw_text, &parsed_response);
        }
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

    return Ok(());
}

fn create_content_with_snapshot(raw_text: &str, response: &SnapResponse) -> String {
    let parts_of_file: Vec<&str> = raw_text.split("SNAPSHOT").collect();
    let mut file_appending = "SNAPSHOT\nstatus: ".to_owned() + &response.status.to_string();

    if response.options.include_headers {
        file_appending += "\n\n";
        for (name, value) in &response.headers {
            file_appending += &(name.as_str().to_owned() + ": " + value.to_str().unwrap());
            file_appending += "\n";
        }
    } else {
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
