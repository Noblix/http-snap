use crate::types::SnapOptions;
use chumsky::error::Simple;
use chumsky::prelude::{just};
use chumsky::text::whitespace;
use chumsky::Parser;

pub(crate) fn options_parser() -> impl Parser<char, SnapOptions, Error = Simple<char>> {
    let repeated_spaces = just(' ').repeated();

    let no_options = whitespace().map(|_| SnapOptions {
        include_headers: true,
    });

    let include_headers = whitespace()
        .then(just("include-headers"))
        .then(repeated_spaces.clone())
        .then(just(":"))
        .then(repeated_spaces.clone())
        .ignore_then(just("true").or(just("false")))
        .map(|value| if value == "true" { true } else { false })
        .map(|value| SnapOptions {
            include_headers: value,
        });

    return include_headers.or(no_options);
}
