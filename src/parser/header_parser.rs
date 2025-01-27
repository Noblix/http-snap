use std::ops::Add;
use crate::types::*;
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;

pub(crate) fn headers_parser() -> impl Parser<char, Vec<Header>, Error = Simple<char>> {
    let repeated_spaces = just(' ').repeated();

    let header_value = filter(|x: &char| x != &'\r' && x != &'\n' && x != &'{' && x != &'}')
        .repeated()
        .at_least(1)
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());

    let header_key = text::ident()
        .then(filter(|c: &char| c.is_ascii_alphanumeric() || c == &'-').repeated())
        .foldl(|start, c| start.add(&c.to_string()));

    let headers = (header_key
        .then_ignore(just(':'))
        .then_ignore(repeated_spaces)
        .then(header_value))
        .map(|(name, value)| Header { name, value })
        .padded()
        .repeated();

    return headers;
}