use crate::parser::snapshot_parser::ignore_comparison_parser;
use crate::parser::variable_parser::variable_store_parser;
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
            .then(variable_store_parser().or_not())
            .then_ignore(whitespace())
            .map(|((name, value), variable_store)| Header {
                name,
                value,
                variable_store,
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
                (ignore_comparison_parser().map(|_| (String::new(), Some(Comparison::Ignore))))
                    .or(header_value().map(|value| (value, Some(Comparison::Exact)))),
            )
            .then(variable_store_parser().or_not())
            .then_ignore(whitespace())
            .map(|((name, (value, comparison)), variable_store)| Header {
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

fn header_value() -> impl Parser<char, String, Error = Simple<char>> {
    return filter(|x: &char| x != &'\r' && x != &'\n' && x != &'{' && x != &'}')
        .repeated()
        .at_least(1)
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());
}
