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

fn parser() -> impl Parser<char, HttpFile, Error = Simple<char>> {
    let base = option_parser::options_parser()
        .then_ignore(variable_parser::variables_skipper())
        .then(url_parser::verb_parser())
        .then(url_parser::url_parser())
        .then(header_parser::headers_parser())
        .then(body_parser::body_parser())
        .then(snapshot_parser::snapshot_parser())
        .map(|(((((options, verb), url), headers), body), snapshot)| HttpFile {
            options,
            verb,
            url,
            headers,
            body,
            snapshot,
        });

    return base;
}

pub fn parse_file(input: &str, previous: Option<&SnapResponse>) -> Result<HttpFile, Vec<Simple<char>>> {
    let without_variables = variable_parser::replace_variables(input, previous);
    let result = parser().parse(without_variables);
    return result;
}

pub async fn parse_response(
    options: SnapOptions,
    response: &HttpResponse,
) -> Result<SnapResponse, Box<dyn std::error::Error>> {
    let body = body_parser::body_parser()
        .parse(response.body.clone())
        .unwrap();
    return Ok(SnapResponse {
        options,
        status: response.status,
        headers: response.headers.clone(),
        body,
    });
}
