use crate::parser::body_parser;
use crate::types::{CompositeString, HttpVerb};
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
    let url = body_parser::characters_parser();
    return url;
}
