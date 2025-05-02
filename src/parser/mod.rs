mod body_parser;
mod header_parser;
mod snapshot_parser;
mod url_parser;
mod variable_parser;

use crate::client::HttpResponse;
use crate::types::*;
use chumsky::error::Simple;
use chumsky::Parser;
use std::collections::HashMap;

fn parser() -> impl Parser<char, HttpFile, Error = Simple<char>> {
    let base = variable_parser::variables_parser(false)
        .then(url_parser::verb_parser())
        .then(url_parser::url_parser())
        .then(header_parser::headers_parser(false))
        .then(body_parser::body_parser(false))
        .then(snapshot_parser::snapshots_parser())
        .map(
            |(((((variables, verb), url), headers), body), snapshots)| HttpFile {
                variables,
                verb,
                url,
                headers,
                body,
                snapshots,
            },
        );

    return base;
}

pub fn parse_file(input: &str) -> Result<HttpFile, Vec<Simple<char>>> {
    let result = parser().parse(input);
    return result;
}

pub async fn parse_response(
    response: &HttpResponse,
) -> Result<SnapResponse, Box<dyn std::error::Error>> {
    let body = body_parser::body_parser(false)
        .parse(response.body.clone())
        .unwrap();
    let headers = response
        .headers
        .iter()
        .map(|(key, value)| {
            let name = key.as_str().to_string();
            let header = Header {
                name: name.clone(),
                value: CompositeString::new(vec![CompositeStringPart::Literal(
                    value.to_str().unwrap().to_string(),
                )]),
                variable_store: None,
                comparison: None,
            };
            (name, header)
        })
        .collect();
    return Ok(SnapResponse {
        status: response.status,
        headers,
        body,
    });
}

pub fn parse_environment(input: &str) -> Result<HashMap<String, Variable>, Vec<Simple<char>>> {
    return variable_parser::variables_parser(false)
        .map(|vars| vars)
        .parse(input);
}
