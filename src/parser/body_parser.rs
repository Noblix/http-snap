﻿use crate::parser::variable_parser::{
    variable_name_parser, variable_name_string_parser, variable_store_body_parser,
};
use crate::types::*;
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::{digits, int, whitespace};
use chumsky::Parser;
use std::rc::Rc;

// Based on https://www.json.org/json-en.html
pub(crate) fn body_parser(
    comparison: bool,
) -> impl Parser<char, Option<Json>, Error = Simple<char>> {
    let no_body = empty().map(|_| None);

    return json_parser(comparison).map(|body| Some(body)).or(no_body);
}

fn json_parser(comparison: bool) -> impl Parser<char, Json, Error = Simple<char>> {
    return element_parser(comparison).map(|element| Json { element });
}

pub(crate) fn value_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, Value, Error = Simple<char>> {
    let variable = variable_name_string_parser().map(|name| Value::VariableReference(name));
    let boolean = (just("true").map(|_| Value::Boolean(true)))
        .or(just("false").map(|_| Value::Boolean(false)));
    let null = just("null").map(|_| Value::Null());

    return object_parser(element_parser.clone())
        .or(array_parser(element_parser.clone()))
        .or(string_value_parser())
        .or(number_value_parser())
        .or(boolean)
        .or(null)
        .or(variable);
}

fn object_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, Value, Error = Simple<char>> {
    let empty = whitespace().delimited_by(just("{"), just("}")).map(|_| {
        Value::Object(Object {
            members: Vec::new(),
        })
    });

    let members = members_parser(element_parser)
        .delimited_by(just("{"), just("}"))
        .map(|members| Value::Object(Object { members }));

    return empty.or(members);
}

fn members_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, Vec<Member>, Error = Simple<char>> {
    return member_parser(element_parser)
        .separated_by(just(","))
        .collect();
}

fn member_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>>,
) -> impl Parser<char, Member, Error = Simple<char>> {
    return whitespace()
        .ignore_then(member_key_parser())
        .then_ignore(whitespace())
        .then_ignore(just(":"))
        .then(element_parser)
        .map(|(key, value)| Member { key, value });
}

fn member_key_parser() -> impl Parser<char, String, Error = Simple<char>> {
    return character_parser()
        .repeated()
        .at_least(1)
        .delimited_by(just('"'), just('"'))
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());
}

fn array_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, Value, Error = Simple<char>> {
    let empty = whitespace()
        .delimited_by(just("["), just("]"))
        .map(|_| Value::Array(Array::Literal(Vec::new())));

    let literal = elements_parser(element_parser.clone())
        .delimited_by(just("["), just("]"))
        .map(|elements| Value::Array(Array::Literal(elements)));

    let dots = whitespace().then(just("...")).then(whitespace());
    let starts_with = elements_parser(element_parser.clone())
        .then_ignore(dots)
        .delimited_by(just("["), just("]"))
        .map(|elements| Value::Array(Array::StartsWith(elements)));

    let contains = dots
        .ignore_then(elements_parser(element_parser.clone()))
        .then_ignore(dots)
        .delimited_by(just("["), just("]"))
        .map(|elements| Value::Array(Array::Contains(elements)));

    let ends_with = dots
        .ignore_then(elements_parser(element_parser.clone()))
        .delimited_by(just("["), just("]"))
        .map(|elements| Value::Array(Array::EndsWith(elements)));

    return choice((empty, literal, starts_with, contains, ends_with));
}

fn elements_parser(
    element_parser: impl Parser<char, Element, Error = Simple<char>> + Clone,
) -> impl Parser<char, Vec<Element>, Error = Simple<char>> {
    return element_parser.separated_by(just(",")).collect();
}

pub(crate) fn element_parser(
    compare: bool,
) -> impl Parser<char, Element, Error = Simple<char>> + Clone {
    return if compare {
        element_compare_parser()
    } else {
        element_no_compare_parser()
    };
}

fn element_compare_parser() -> Rc<dyn Parser<char, Element, Error = Simple<char>>> {
    return Rc::new(recursive(|element_compare_parser| {
        whitespace()
            .ignore_then(choice((
                variable_store_body_parser(element_compare_parser.clone()).map(
                    |(variable_store, (value, comparison))| Element {
                        value,
                        variable_store,
                        comparison,
                    },
                ),
                value_parser(element_compare_parser).map(|value| Element {
                    value,
                    variable_store: None,
                    comparison: Some(Comparison::Exact),
                }),
            )))
            .then_ignore(whitespace())
    }));
}

fn element_no_compare_parser() -> Rc<dyn Parser<char, Element, Error = Simple<char>>> {
    return Rc::new(recursive(|element_no_compare_parser| {
        whitespace()
            .ignore_then(value_parser(element_no_compare_parser))
            .then_ignore(whitespace())
            .then_ignore(whitespace())
            .map(|value| Element {
                value,
                variable_store: None,
                comparison: None,
            })
    }));
}

fn string_value_parser() -> impl Parser<char, Value, Error = Simple<char>> {
    return characters_parser()
        .delimited_by(just('"'), just('"'))
        .map(|val| Value::String(val));
}

pub(crate) fn characters_parser() -> impl Parser<char, CompositeString, Error = Simple<char>> {
    return variable_name_parser()
        .or(character_parser().map(|c| CompositeStringPart::Literal(c.to_string())))
        .repeated()
        .map(|parts| {
            let merged_parts = CompositeStringPart::merge_literals(parts);
            CompositeString {
                parts: merged_parts,
            }
        });
}

fn character_parser() -> impl Parser<char, char, Error = Simple<char>> {
    let valid_char = filter(|c: &char| {
        let in_range = *c >= '\u{0020}' && *c <= '\u{10FFF}';
        let is_special = *c == '"' || *c == '\\';
        in_range && !is_special
    })
    .map(|c| c);

    return escape_parser().or(valid_char);
}

fn escape_parser() -> impl Parser<char, char, Error = Simple<char>> {
    return just("\\").ignore_then(choice((
        just('"').to('"'),
        just("\\").to('\\'),
        just("/").to('/'),
        just("b").to('\u{0008}'), // Backspace
        just("f").to('\u{000C}'), // Form feed
        just("n").to('\n'),
        just("r").to('\r'),
        just("t").to('\t'),
        hex_parser(),
    )));
}

fn hex_parser() -> impl Parser<char, char, Error = Simple<char>> {
    return just("u")
        .ignore_then(one_of("0123456789abcdefABCDEF").repeated().exactly(4))
        .map(|digits: Vec<char>| {
            let as_string: String = digits.into_iter().collect();
            char::from_u32(u32::from_str_radix(&as_string, 16).unwrap()).unwrap()
        });
}

fn number_value_parser() -> impl Parser<char, Value, Error = Simple<char>> {
    return number_parser().map(|value| Value::Number(value));
}

fn number_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    return exponent_parser().or(fraction_parser()).or(integer_parser());
}

fn integer_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    let positive = int(10).map(|num: String| num.parse::<i64>().unwrap());
    let negative = just("-").ignore_then(positive).map(|num| -num);
    return positive.or(negative).map(|num| Number::Int(num));
}

fn fraction_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    let positive = int(10)
        .then_ignore(just("."))
        .then(digits(10))
        .map(|(whole, frac): (String, String)| format!("{whole}.{frac}"));
    let negative = just("-").ignore_then(positive).map(|num| format!("-{num}"));
    return positive.or(negative).map(|num| Number::Fraction(num));
}

fn exponent_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    let sign = (one_of("+-").or_not()).map(|c: Option<char>| {
        if c.is_none() {
            "".to_string()
        } else {
            c.unwrap().to_string()
        }
    });
    let digits = one_of("0123456789")
        .repeated()
        .at_least(1)
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());

    return (sign.clone())
        .then(digits.clone())
        .then(just("."))
        .then(digits.clone())
        .then(one_of("eE"))
        .then(sign.clone())
        .then(digits.clone())
        .map(
            |((((((base_sign, base), dot), decimals), e_lit), exponent_sign), exponent)| {
                Number::Exponent(
                    [
                        base_sign,
                        base,
                        dot.to_string(),
                        decimals,
                        e_lit.to_string(),
                        exponent_sign.to_string(),
                        exponent,
                    ]
                    .concat(),
                )
            },
        );
}
