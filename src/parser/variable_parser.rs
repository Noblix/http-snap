use crate::parser::body_parser::{element_parser, value_parser};
use crate::types::{CompositeStringPart, Value};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::{whitespace, Character};
use chumsky::Parser;
use std::collections::HashMap;

pub(crate) fn variables_parser(comparison: bool) -> impl Parser<char, HashMap<String, Value>, Error = Simple<char>>
{
    return just('@')
        .ignore_then(text::ident())
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then_ignore(just('='))
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then(value_parser(element_parser(comparison)))
        .padded()
        .repeated()
        .map(|vars: Vec<(String, Value)>| {
            vars.into_iter()
                .map(|(name, value)| (name, value))
                .collect::<HashMap<String, Value>>()
        });
}

pub(crate) fn variable_store_parser(
) -> impl Parser<char, String, Error = Simple<char>> {
    return just("->")
        .ignore_then(whitespace())
        .ignore_then(just('@'))
        .ignore_then(text::ident())
        .map(|name| name);
}

pub(crate) fn variable_name_parser() -> impl Parser<char, CompositeStringPart, Error = Simple<char>>
{
    return variable_reference_parser().map(|name| CompositeStringPart::VariableName(name));
}

pub(crate) fn variable_name_string_parser() -> impl Parser<char, String, Error = Simple<char>> {
    return variable_reference_parser();
}

fn variable_reference_parser() -> impl Parser<char, String, Error = Simple<char>> {
    return just("{{")
        .ignore_then(text::ident())
        .then_ignore(just("}}"))
        .map(|name| name);
}
