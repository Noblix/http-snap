use crate::parser::body_parser::{body_parser, characters_parser};
use crate::parser::header_parser::headers_parser;
use crate::types::{Comparison, Number, Snapshot};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::whitespace;
use chumsky::Parser;

pub(crate) fn snapshots_parser() -> impl Parser<char, Vec<Snapshot>, Error = Simple<char>> {
    let no_snapshot = end().map(|_| Vec::new());

    let snapshot = status_parser()
        .then_ignore(whitespace())
        .then(headers_parser(true))
        .then_ignore(whitespace())
        .then(body_parser(true))
        .map(|((status, headers), body)| Snapshot {
            status,
            headers,
            body,
        });

    let snapshots = just("SNAPSHOT").ignore_then(whitespace()).ignore_then(
        snapshot
            .padded()
            .separated_by(just("||"))
            .then_ignore(whitespace().then(end()))
            .map(|s| s),
    );

    return whitespace().ignore_then(no_snapshot.or(snapshots));
}

pub(crate) fn ignore_comparison_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("_"))
        .then_ignore(whitespace())
        .to(Comparison::Ignore);
}

pub(crate) fn timestamp_format_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("timestamp"))
        .ignore_then(characters_parser().delimited_by(just("(\""), just("\")")))
        .map(|pattern| Comparison::TimestampFormat(pattern));
}

pub(crate) fn guid_format_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("guid"))
        .then_ignore(whitespace())
        .to(Comparison::Guid);
}

fn status_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    return just("status: ")
        .ignore_then(one_of("0123456789").repeated().exactly(3))
        .map(|code: Vec<char>| {
            Number::Int(code.into_iter().collect::<String>().parse::<i64>().unwrap())
        });
}
