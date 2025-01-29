use http_snap::{client, parser};
use std::fs::{read_to_string, File};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_to_file = "http-examples/get-simple.http";
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let http_file = parser::parse_file(text).unwrap();
    println!("{:?}", http_file);

    let parsed_response = client::send_request(&http_file).await?;

    let parts_of_file: Vec<&str> = raw_text.split("SNAPSHOT:").collect();
    let file_appending = "SNAPSHOT:\nstatus: ".to_owned()
        + &parsed_response.status.to_string()
        + "\n"
        + &serde_json::to_string_pretty(&parsed_response.body).unwrap();

    let merged = match parts_of_file.len() {
        1 => raw_text + "\n\n" + &file_appending,
        2 => parts_of_file[0].to_owned() + &file_appending,
        _ => panic!("Found more than one snapshot place"),
    };

    let mut file = File::options()
        .write(true)
        .truncate(true)
        .open(path_to_file)?;
    file.write_all(&merged.as_bytes())
        .expect("Unable to write snapshot");
    file.flush()?;

    return Ok(());
}
