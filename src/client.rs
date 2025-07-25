﻿use crate::types::{ClientOptions, Header, HttpFile, HttpVerb, Json};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Client, Method};

pub struct HttpClient {
    client: Client,
    options: ClientOptions
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: String,
}

impl HttpClient {
    pub fn new(options: &ClientOptions) -> Self {
        let client = Client::builder()
            .cookie_store(options.use_cookies.unwrap_or(true))
            .build()
            .expect("Failed to build client");
        return Self { client, options: options.clone() };
    }

    pub async fn send_request(
        &self,
        http_file: &HttpFile,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let headers = self.get_headers(&http_file.headers);
        let body = get_json(&http_file.body);

        let url = &http_file.url.to_string();
        let verb_setup = match http_file.verb {
            HttpVerb::CONNECT => self.client.request(Method::CONNECT, url),
            HttpVerb::DELETE => self.client.delete(url),
            HttpVerb::GET => self.client.get(url),
            HttpVerb::HEAD => self.client.head(url),
            HttpVerb::OPTIONS => self.client.request(Method::OPTIONS, url),
            HttpVerb::PATCH => self.client.patch(url),
            HttpVerb::POST => self.client.post(url),
            HttpVerb::PUT => self.client.put(url),
            HttpVerb::TRACE => self.client.request(Method::TRACE, url),
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

    fn get_headers(&self, request_headers: &Vec<Header>) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if let Some(default_headers) = &self.options.default_headers {
            for header in default_headers {
                headers.insert(
                    HeaderName::from_bytes(header.name.as_ref()).unwrap(),
                    header.value.parse().unwrap(),
                );
            }
        }

        for header in request_headers {
            headers.insert(
                HeaderName::from_bytes(header.name.as_ref()).unwrap(),
                header.value.to_string().parse().unwrap(),
            );
        }
        return headers;
    }
}

fn get_json(body: &Option<Json>) -> String {
    return serde_json::to_string(body).unwrap();
}
