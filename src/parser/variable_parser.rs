use crate::types::SnapResponse;
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;
use regex::Regex;

pub(crate) fn replace_variables(input: &str, previous: Option<&SnapResponse>) -> String {
    let local_replaced = replace_local_variables(input);
    
    if let Some(prev) = previous {
        let previous_headers_replaced = replace_previous_header_variables(&local_replaced, prev);
        return previous_headers_replaced;
    }

    return local_replaced;
}

pub(crate) fn variables_skipper() -> impl Parser<char, String, Error = Simple<char>> {
    return just('@')
        .then(text::ident())
        .then(just(' ').repeated())
        .then(just('='))
        .then(just(' ').repeated())
        .then(filter(|x: &char| !x.is_whitespace()).repeated().at_least(1))
        .padded()
        .repeated()
        .ignored()
        .map(|_| "IGNORE ME".to_string());
}

fn replace_local_variables(input: &str) -> String {
    let mut result: String = input.to_string();

    for mut line in input.lines() {
        line = line.trim();
        if line.starts_with("@") {
            let parts: Vec<&str> = line.split("=").collect();
            if parts.len() >= 2 {
                let value = parts[1..].concat();
                let replace = ["{{", &parts[0][1..].trim(), "}}"].concat();
                result = result.replace(&replace, &value.trim());
            }
        }

        let verb_options = ["GET", "PATCH", "POST", "PUT"];
        for prefix in verb_options {
            if line.starts_with(prefix) {
                break;
            }
        }
    }

    return result;
}

fn replace_previous_header_variables(input: &str, previous: &SnapResponse) -> String {
    let mut result: String = input.to_string();

    let header_regex = Regex::new(r#"\{\{previous\.headers\["([a-zA-Z0-9\-\_]+)"\]\}\}"#).unwrap();
    let header_names = header_regex.captures_iter(input).map(|c| c.extract());
    for (_, [header_name]) in header_names {
        let header = previous.headers.get(header_name);
        if let Some(value) = header {
            let replace = ["{{previous.headers[\"", header_name, "\"]}}"].concat();
            result = result.replace(&replace, value.to_str().unwrap().trim())
        }
    }

    return result;
}