mod body_parser;
mod header_parser;
mod url_parser;
mod variable_parser;

use crate::types::*;
use chumsky::error::Simple;
use chumsky::Parser;

fn parser() -> impl Parser<char, HttpFile, Error = Simple<char>> {
    let base = variable_parser::variables_skipper()
        .ignore_then(url_parser::verb_parser())
        .then(url_parser::url_parser())
        .then(header_parser::headers_parser())
        .then(body_parser::body_parser())
        .map(|(((verb, url), headers), body)| HttpFile {
            verb,
            url,
            headers,
            body,
        });

    return base;
}

pub fn parse(input: &str) -> Result<HttpFile, Vec<Simple<char>>> {
    let without_variables = variable_parser::replace_variables(input);
    let result = parser().parse(without_variables);
    return result;
}
