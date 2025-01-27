use std::fs::read_to_string;
use http_snap::parser;
use http_snap::types::*;

use reqwest::header::{HeaderMap, HeaderName};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_text = read_to_string("http-examples/post-small-body.http").unwrap();
    let text = raw_text.trim_start_matches("\u{feff}");
    let result = parser::parse(text);
    let http_file = result.unwrap();
    println!("{:?}", http_file);

    let headers = get_headers(&http_file.headers);
    let body = get_json(&http_file.body);

    let client = reqwest::Client::new();

    if http_file.verb == HttpVerb::GET {
        client.get(&http_file.url).headers(headers).send().await?;
    } else if http_file.verb == HttpVerb::POST {
        client.post(&http_file.url).headers(headers).body(body).send().await?;
    }

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