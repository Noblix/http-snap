use http_snap::parser;
use http_snap::types::*;
use reqwest::header::{HeaderMap, HeaderName};
use std::fs::{read_to_string, File};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path_to_file = "http-examples/post-simple.http";
    let raw_text = read_to_string(path_to_file).unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let result = parser::parse_file(text);
    let http_file = result.unwrap();
    println!("{:?}", http_file);

    let headers = get_headers(&http_file.headers);
    let body = get_json(&http_file.body);

    let client = reqwest::Client::new();

    let response = match http_file.verb {
        HttpVerb::GET => client.get(&http_file.url).headers(headers).send().await?,
        HttpVerb::POST => {
            client
                .post(&http_file.url)
                .headers(headers)
                .body(body)
                .send()
                .await?
        }
        _ => panic!("Unknown verb!"),
    };

    let parsed_response = parser::parse_response(response).await?;

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

fn get_headers(request_headers: &Vec<Header>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for header in request_headers {
        headers.insert(
            HeaderName::from_bytes(header.name.as_ref()).unwrap(),
            header.value.parse().unwrap(),
        );
    }
    return headers;
}

fn get_json(body: &Json) -> String {
    return serde_json::to_string(body).unwrap();
}
