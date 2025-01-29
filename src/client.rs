use crate::parser;
use crate::types::{Header, HttpFile, HttpVerb, Json, SnapResponse};
use reqwest::header::{HeaderMap, HeaderName};

pub async fn send_request(
    http_file: &HttpFile,
) -> Result<SnapResponse, Box<dyn std::error::Error>> {
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

    return Ok(parsed_response);
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
