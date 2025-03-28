mod body_parser;
mod header_parser;
mod option_parser;
mod snapshot_parser;
mod url_parser;
mod variable_parser;

use crate::client::HttpResponse;
use crate::types::*;
use chumsky::error::Simple;
use chumsky::Parser;
use std::collections::HashMap;

fn parser() -> impl Parser<char, HttpFile, Error = Simple<char>> {
    let base = option_parser::options_parser()
        .then(variable_parser::variables_parser(false))
        .then(url_parser::verb_parser())
        .then(url_parser::url_parser())
        .then(header_parser::headers_parser(false))
        .then(body_parser::body_parser(false))
        .then(snapshot_parser::snapshot_parser())
        .map(|((((((options, variables), verb), url), headers), body), snapshot)| HttpFile {
            options,
            variables,
            verb,
            url,
            headers,
            body,
            snapshot,
        });

    return base;
}

pub fn parse_file(input: &str) -> Result<HttpFile, Vec<Simple<char>>> {
    let result = parser().parse(input);
    return result;
}

pub async fn parse_response(
    options: SnapOptions,
    response: &HttpResponse,
) -> Result<SnapResponse, Box<dyn std::error::Error>> {
    let body = body_parser::body_parser(false)
        .parse(response.body.clone())
        .unwrap();
    return Ok(SnapResponse {
        options,
        status: response.status,
        headers: response.headers.clone(),
        body,
    });
}

pub fn parse_environment(input: &str) -> Result<HashMap<String, Value>, Vec<Simple<char>>> {
    return variable_parser::variables_parser(false)
        .map(|vars| vars)
        .parse(input);
}
