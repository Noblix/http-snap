use crate::parser::body_parser::{element_parser, value_parser};
use crate::types::{CompositeStringPart, Value};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::Character;
use chumsky::Parser;
use std::collections::HashMap;

pub(crate) fn variables_parser() -> impl Parser<char, HashMap<String, Value>, Error = Simple<char>>
{
    return just('@')
        .ignore_then(text::ident())
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then_ignore(just('='))
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then(value_parser(element_parser()))
        .padded()
        .repeated()
        .map(|vars: Vec<(String, Value)>| {
            vars.into_iter()
                .map(|(name, value)| (name, value))
                .collect::<HashMap<String, Value>>()
        });
}

pub(crate) fn variable_name_parser() -> impl Parser<char, CompositeStringPart, Error = Simple<char>>
{
    return variable_reference_parser().map(|name| CompositeStringPart::VariableName(name));
}

pub(crate) fn variable_name_string_parser() -> impl Parser<char, String, Error = Simple<char>> {
    return variable_reference_parser();
}

fn variable_reference_parser() -> impl Parser<char, String, Error = Simple<char>> {
    let field_parser = just(".").ignore_then(text::ident().map(|field| format!(".{field}")));
    let look_up_parser = text::int(10)
        .or(text::ident())
        .delimited_by(just("[\""), just("\"]"))
        .map(|key| format!("[\"{key}\"]"));

    return just("{{")
        .ignore_then(text::ident())
        .then((field_parser.or(look_up_parser)).repeated())
        .then_ignore(just("}}"))
        .map(|(head, tail): (String, Vec<String>)| head + &tail.join(""));
}
