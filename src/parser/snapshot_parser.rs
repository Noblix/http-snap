use crate::snapshot_types::Number::Int;
use crate::snapshot_types::*;
use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::text::{int, whitespace};
use chumsky::Parser;
use std::ops::Add;

// Based on https://www.json.org/json-en.html
pub(crate) fn snapshot_parser() -> impl Parser<char, Snapshot, Error = Simple<char>> {
    let no_snapshot = end().map(|_| Snapshot {
        status: Comparison::Exact(ValueComparer::Null()),
        headers: Vec::new(),
        body: JsonComparer {
            element: ElementComparer {
                value: Comparison::Exact(ValueComparer::Null()),
            },
        },
    });

    let snapshot = just("SNAPSHOT:")
        .ignore_then(whitespace())
        .ignore_then(status_parser())
        .then_ignore(whitespace())
        .then(headers_parser())
        .then_ignore(whitespace())
        .then(json_parser())
        .map(|((status, headers), body)| Snapshot {
            status,
            headers,
            body,
        });

    return whitespace().ignore_then(no_snapshot.or(snapshot));
}

fn status_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return just("status: ")
        .ignore_then(one_of("0123456789").repeated().exactly(3))
        .map(|code: Vec<char>| {
            Comparison::Exact(ValueComparer::Number(Int(code
                .into_iter()
                .collect::<String>()
                .parse::<i64>()
                .unwrap())))
        });
}

fn headers_parser() -> impl Parser<char, Vec<HeaderComparer>, Error = Simple<char>> {
    let repeated_spaces = just(' ').repeated();

    let header_value = filter(|x: &char| x != &'\r' && x != &'\n' && x != &'{' && x != &'}')
        .repeated()
        .at_least(1)
        .map(|chars: Vec<char>| Comparison::Exact(ValueComparer::String(chars.into_iter().collect::<String>())));

    let header_key = text::ident()
        .then(filter(|c: &char| c.is_ascii_alphanumeric() || c == &'-').repeated())
        .foldl(|start, c| start.add(&c.to_string()));

    let headers = (header_key
        .then_ignore(just(':'))
        .then_ignore(repeated_spaces)
        .then(ignore_comparison_parser().or(header_value)))
    .map(|(name, value)| HeaderComparer {
        name,
        value,
    })
    .padded()
    .repeated();

    return headers;
}

fn json_parser() -> impl Parser<char, JsonComparer, Error = Simple<char>> {
    return element_parser().map(|element| JsonComparer { element });
}

fn value_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>> + Clone,
) -> impl Parser<char, Comparison, Error = Simple<char>> {
    let number = number_parser().map(|value| Comparison::Exact(ValueComparer::Number(value)));
    let boolean = (just("true").map(|_| Comparison::Exact(ValueComparer::Boolean(true))))
        .or(just("false").map(|_| Comparison::Exact(ValueComparer::Boolean(false))));
    let null = just("null").map(|_| Comparison::Exact(ValueComparer::Null()));

    return ignore_comparison_parser()
        .or(object_parser(element_parser.clone()))
        .or(array_parser(element_parser.clone()))
        .or(string_value_parser())
        .or(number)
        .or(boolean)
        .or(null);
}

fn ignore_comparison_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return whitespace()
        .then(just("_"))
        .then_ignore(whitespace())
        .map(|_| Comparison::Ignore);
}

fn object_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>> + Clone,
) -> impl Parser<char, Comparison, Error = Simple<char>> {
    let empty = whitespace().delimited_by(just("{"), just("}")).map(|_| {
        Comparison::Exact(ValueComparer::Object(ObjectComparer {
            members: Vec::new(),
        }))
    });

    let members = members_parser(element_parser)
        .delimited_by(just("{"), just("}"))
        .map(|members| Comparison::Exact(ValueComparer::Object(ObjectComparer { members })));

    return empty.or(members);
}

fn members_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>> + Clone,
) -> impl Parser<char, Vec<MemberComparer>, Error = Simple<char>> {
    return member_parser(element_parser)
        .separated_by(just(","))
        .collect();
}

fn member_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>>,
) -> impl Parser<char, MemberComparer, Error = Simple<char>> {
    return whitespace()
        .ignore_then(member_key_parser())
        .then_ignore(whitespace())
        .then_ignore(just(":"))
        .then(element_parser)
        .map(|(key, value)| MemberComparer { key, value });
}

fn member_key_parser() -> impl Parser<char, String, Error = Simple<char>> {
    return characters_parser(1).delimited_by(just('"'), just('"'));
}

fn array_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>> + Clone,
) -> impl Parser<char, Comparison, Error = Simple<char>> {
    let empty = whitespace().delimited_by(just("["), just("]")).map(|_| {
        Comparison::Exact(ValueComparer::Array(ArrayComparer {
            elements: Vec::new(),
        }))
    });

    let elements = elements_parser(element_parser)
        .delimited_by(just("["), just("]"))
        .map(|elements| Comparison::Exact(ValueComparer::Array(ArrayComparer { elements })));

    return empty.or(elements);
}

fn elements_parser(
    element_parser: impl Parser<char, ElementComparer, Error = Simple<char>> + Clone,
) -> impl Parser<char, Vec<ElementComparer>, Error = Simple<char>> {
    return element_parser.separated_by(just(",")).collect();
}

fn element_parser() -> impl Parser<char, ElementComparer, Error = Simple<char>> {
    return recursive(|element_parser| {
        whitespace()
            .ignore_then(value_parser(element_parser))
            .then_ignore(whitespace())
            .map(|value| ElementComparer { value })
    });
}

fn string_value_parser() -> impl Parser<char, Comparison, Error = Simple<char>> {
    return characters_parser(0)
        .delimited_by(just('"'), just('"'))
        .map(|c| Comparison::Exact(ValueComparer::String(c.to_string())));
}

fn characters_parser(minimum_repeat: usize) -> impl Parser<char, String, Error = Simple<char>> {
    return character_parser()
        .repeated()
        .at_least(minimum_repeat)
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());
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
        .then(int(10))
        .map(|(whole, frac): (String, String)| format!("{whole}.{frac}").parse::<f64>().unwrap());
    let negative = just("-").ignore_then(positive).map(|num| -num);
    return positive.or(negative).map(|num| Number::Fraction(num));
}

fn exponent_parser() -> impl Parser<char, Number, Error = Simple<char>> {
    let digits = one_of("0123456789")
        .repeated()
        .at_least(1)
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());

    return (just("-").or(empty().to("")))
        .then(digits.clone())
        .then(just("."))
        .then(digits.clone())
        .then(one_of("eE"))
        .then(one_of("+-"))
        .then(digits.clone())
        .map(
            |((((((base_sign, base), dot), decimals), e_lit), exponent_sign), exponent)| {
                Number::Exponent(
                    [
                        base_sign,
                        &base,
                        dot,
                        &decimals,
                        &e_lit.to_string(),
                        &exponent_sign.to_string(),
                        &exponent,
                    ]
                    .concat(),
                )
            },
        );
}
