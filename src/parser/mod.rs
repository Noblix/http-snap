mod body_parser;
mod header_parser;
mod url_parser;
mod variable_parser;
mod snapshot_parser;

use crate::types::*;
use chumsky::error::Simple;
use chumsky::Parser;
use reqwest::Response;

fn parser() -> impl Parser<char, HttpFile, Error = Simple<char>> {
    let base = variable_parser::variables_skipper()
        .ignore_then(url_parser::verb_parser())
        .then(url_parser::url_parser())
        .then(header_parser::headers_parser())
        .then(body_parser::body_parser())
        .then(snapshot_parser::snapshot_parser())
        .map(|((((verb, url), headers), body), snapshot)| HttpFile {
            verb,
            url,
            headers,
            body,
            snapshot,
        });

    return base;
}

pub fn parse_file(input: &str) -> Result<HttpFile, Vec<Simple<char>>> {
    let without_variables = variable_parser::replace_variables(input);
    let result = parser().parse(without_variables);
    return result;
}

pub async fn parse_response(
    response: Response,
) -> Result<SnapResponse, Box<dyn std::error::Error>> {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let raw_body = response.text().await?;
    let body = body_parser::body_parser().parse(raw_body).unwrap();
    return Ok(SnapResponse {
        status,
        headers,
        body,
    });
}
