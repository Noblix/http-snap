use crate::parser::variable_parser::{variable_name_parser, variable_store_header_parser};
use crate::types::*;
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::whitespace;
use chumsky::Parser;
use std::ops::Add;

pub(crate) fn headers_parser(
    compare: bool,
) -> impl Parser<char, Vec<Header>, Error = Simple<char>> {
    if compare {
        return headers_compare_parser();
    }

    return headers_no_compare_parser();
}

fn headers_no_compare_parser() -> Box<dyn Parser<char, Vec<Header>, Error = Simple<char>>> {
    return Box::new(
        header_key()
            .then_ignore(just(':'))
            .then_ignore(repeated_spaces())
            .then(header_value())
            .then_ignore(whitespace())
            .map(|(name, value)| Header {
                name,
                value,
                variable_store: None,
                comparison: None,
            })
            .padded()
            .repeated(),
    );
}

fn headers_compare_parser() -> Box<dyn Parser<char, Vec<Header>, Error = Simple<char>>> {
    return Box::new(
        header_key()
            .then_ignore(just(':'))
            .then_ignore(repeated_spaces())
            .then(
                variable_store_header_parser()
                    .or(header_value().map(|value| (None, (value, Some(Comparison::Exact))))),
            )
            .then_ignore(whitespace())
            .map(|(name, (variable_store, (value, comparison)))| Header {
                name,
                value,
                variable_store,
                comparison,
            })
            .padded()
            .repeated(),
    );
}

fn repeated_spaces() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    return just(' ').repeated();
}

fn header_key() -> impl Parser<char, String, Error = Simple<char>> {
    return text::ident()
        .then(filter(|c: &char| c.is_ascii_alphanumeric() || c == &'-').repeated())
        .foldl(|start, c| start.add(&c.to_string()));
}

fn header_value() -> impl Parser<char, CompositeString, Error = Simple<char>> {
    return variable_name_parser()
        .or(filter(|c: &char| *c != '\n').map(|c| CompositeStringPart::Literal(c.to_string())))
        .repeated()
        .map(|parts| {
            let merged_parts = CompositeStringPart::merge_literals(parts);
            CompositeString {
                parts: merged_parts,
            }
        });
}
