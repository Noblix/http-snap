use crate::parser::body_parser::{body_parser, characters_parser};
use crate::parser::header_parser::headers_parser;
use crate::types::{Comparison, Element, Json, Number, Snapshot, Value};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::whitespace;
use chumsky::Parser;

// Based on https://www.json.org/json-en.html
pub(crate) fn snapshot_parser() -> impl Parser<char, Snapshot, Error = Simple<char>> {
    let no_snapshot = end().map(|_| Snapshot {
        status: Number::Int(-1),
        headers: Vec::new(),
        body: Json {
            element: Element {
                value: Value::Null(),
                variable_store: None,
                comparison: Some(Comparison::Ignore),
            },
        },
    });

    let snapshot = just("SNAPSHOT")
        .ignore_then(whitespace())
        .ignore_then(status_parser())
        .then_ignore(whitespace())
        .then(headers_parser(true))
        .then_ignore(whitespace())
        .then(body_parser(true))
        .then_ignore(whitespace().then(end()))
        .map(|((status, headers), body)| Snapshot {
            status,
            headers,
            body,
        });

    return whitespace().ignore_then(no_snapshot.or(snapshot));
}

pub(crate) fn ignore_comparison_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("_"))
        .then_ignore(whitespace())
        .map(|_| Comparison::Ignore);
}

pub(crate) fn timestamp_format_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("timestamp"))
        .ignore_then(characters_parser().delimited_by(just("(\""), just("\")")))
        .map(|pattern| Comparison::TimestampFormat(pattern));
}

fn status_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    return just("status: ")
        .ignore_then(one_of("0123456789").repeated().exactly(3))
        .map(|code: Vec<char>| {
            Number::Int(code
                .into_iter()
                .collect::<String>()
                .parse::<i64>()
                .unwrap())
        });
}