use crate::parser::variable_parser::variable_name_parser;
use crate::types::{CompositeString, CompositeStringPart, HttpVerb};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;

pub(crate) fn verb_parser() -> impl Parser<char, HttpVerb, Error = Simple<char>> {
    let verb = (just("GET").map(|_| HttpVerb::GET))
        .or(just("DELETE").map(|_| HttpVerb::DELETE))
        .or(just("PATCH").map(|_| HttpVerb::PATCH))
        .or(just("POST").map(|_| HttpVerb::POST))
        .or(just("PUT").map(|_| HttpVerb::PUT))
        .padded();

    return verb;
}

pub(crate) fn url_parser() -> impl Parser<char, CompositeString, Error = Simple<char>> {
    let url = (variable_name_parser().or(filter(|x: &char| !x.is_whitespace())
        .repeated()
        .at_least(1)
        .map(|chars| CompositeStringPart::Literal(chars.iter().collect()))))
    .repeated()
    .at_least(1)
    .map(|parts| CompositeString { parts });

    return url;
}
