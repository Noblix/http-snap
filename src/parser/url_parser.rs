use crate::parser::body_parser;
use crate::types::{CompositeString, HttpVerb};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;

pub(crate) fn verb_parser() -> impl Parser<char, HttpVerb, Error = Simple<char>> {
    let verb = (just("CONNECT").map(|_| HttpVerb::CONNECT))
        .or(just("DELETE").map(|_| HttpVerb::DELETE))
        .or(just("GET").map(|_| HttpVerb::GET))
        .or(just("HEAD").map(|_| HttpVerb::HEAD))
        .or(just("OPTIONS").map(|_| HttpVerb::OPTIONS))
        .or(just("PATCH").map(|_| HttpVerb::PATCH))
        .or(just("POST").map(|_| HttpVerb::POST))
        .or(just("PUT").map(|_| HttpVerb::PUT))
        .or(just("TRACE").map(|_| HttpVerb::TRACE))
        .padded();

    return verb;
}

pub(crate) fn url_parser() -> impl Parser<char, CompositeString, Error = Simple<char>> {
    let url = body_parser::characters_parser();
    return url;
}
