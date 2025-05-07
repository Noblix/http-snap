use crate::parser::body_parser::{characters_parser, element_parser, value_parser};
use crate::parser::snapshot_parser::{
    guid_format_parser, ignore_comparison_parser, timestamp_format_parser,
};
use crate::types::{
    Comparison, CompositeString, CompositeStringPart, Element, Generator, Value, Variable,
};
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::Character;
use chumsky::Parser;
use std::collections::HashMap;

pub(crate) fn variables_parser(
    comparison: bool,
) -> impl Parser<char, HashMap<String, Variable>, Error = Simple<char>> {
    return just('@')
        .ignore_then(text::ident())
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then_ignore(just('='))
        .then_ignore(filter(|c: &char| c.is_inline_whitespace()).repeated())
        .then(choice((
            value_parser(element_parser(comparison)).map(|val| Variable::Value(val)),
            generator_parser().map(|generator| Variable::Generator(generator)),
        )))
        .padded()
        .repeated()
        .map(|vars: Vec<(String, Variable)>| {
            vars.into_iter()
                .map(|(name, value)| (name, value))
                .collect::<HashMap<String, Variable>>()
        });
}

fn repeated_spaces() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    return just(' ').repeated();
}

pub(crate) fn variable_store_header_parser(
) -> impl Parser<char, (Option<String>, (CompositeString, Option<Comparison>)), Error = Simple<char>>
{
    return just("{{").then(repeated_spaces()).ignore_then(
        text::ident()
            .map(|name| Some(name))
            .or(just("_").to(None))
            .then_ignore(repeated_spaces().then(just(":").then(repeated_spaces())))
            .then(choice((
                timestamp_format_parser(),
                guid_format_parser(),
                ignore_comparison_parser(),
            )))
            .then(
                just(":")
                    .ignore_then(
                        repeated_spaces()
                            .ignore_then(characters_parser().delimited_by(just("\""), just("\""))),
                    )
                    .or_not(),
            )
            .then_ignore(repeated_spaces().then(just("}}")))
            .map(|((variable_store, comparison), value)| {
                (
                    variable_store,
                    (
                        value.unwrap_or(CompositeString::new(Vec::new())),
                        Some(comparison),
                    ),
                )
            }),
    );
}

pub(crate) fn variable_store_body_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, (Option<String>, (Value, Option<Comparison>)), Error = Simple<char>> {
    return just("{{").padded().ignore_then(
        text::ident()
            .map(|name| Some(name))
            .or(just("_").to(None))
            .then_ignore(just(":").padded())
            .then(
                choice((
                    timestamp_format_parser(),
                    guid_format_parser(),
                    ignore_comparison_parser(),
                ))
                .then(
                    just(":")
                        .padded()
                        .ignore_then(value_parser(element_parser.clone()))
                        .or_not(),
                )
                .map(|(comparison, value)| (value.unwrap_or(Value::Null()), comparison))
                .or(value_parser(element_parser).map(|value| (value, Comparison::Exact))),
            )
            .then_ignore(just("}}").padded())
            .map(|(variable_store, (value, comparison))| {
                (variable_store, (value, Some(comparison)))
            }),
    );
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

fn generator_parser() -> impl Parser<char, Generator, Error = Simple<char>> {
    return just("gen(guid)").to(Generator::Guid);
}
