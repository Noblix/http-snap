use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;

pub(crate) fn replace_variables(input: &str) -> String {
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

        let verb_options = ["GET", "POST", "PATCH"];
        for prefix in verb_options {
            if line.starts_with(prefix) {
                break;
            }
        }
    }

    return result;
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