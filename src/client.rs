use crate::types::{Header, HttpFile, HttpVerb, Json};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Client;

pub struct HttpClient {
    client: Client,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: String,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build client");
        return Self { client };
    }

    pub async fn send_request(
        &self,
        http_file: &HttpFile,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let headers = get_headers(&http_file.headers);
        let body = get_json(&http_file.body);

        let verb_setup = match http_file.verb {
            HttpVerb::GET => self.client.get(&http_file.url),
            HttpVerb::DELETE => self.client.delete(&http_file.url),
            HttpVerb::PATCH => self.client.patch(&http_file.url),
            HttpVerb::POST => self.client.post(&http_file.url),
            HttpVerb::PUT => self.client.put(&http_file.url),
        };
        let response = verb_setup.headers(headers).body(body).send().await?;

        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let body = response.text().await?;
        return Ok(HttpResponse {
            status,
            headers,
            body,
        });
    }
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
