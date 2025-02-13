use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;
use crate::types::HttpVerb;

pub(crate) fn verb_parser() -> impl Parser<char, HttpVerb, Error = Simple<char>> {
    let verb = (just("GET").map(|_| HttpVerb::GET))
        .or(just("DELETE").map(|_| HttpVerb::DELETE))
        .or(just("PATCH").map(|_| HttpVerb::PATCH))
        .or(just("POST").map(|_| HttpVerb::POST))
        .or(just("PUT").map(|_| HttpVerb::PUT))
        .padded();

    return verb;
}

pub(crate) fn url_parser() -> impl Parser<char, String, Error = Simple<char>> {
    let url = filter(|x: &char| !x.is_whitespace())
        .repeated()
        .at_least(1)
        .collect();

    return url;
}