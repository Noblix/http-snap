use chumsky::error::Simple;
use chumsky::prelude::*;
use chumsky::Parser;

pub(crate) fn variables_skipper() -> impl Parser<char, String, Error = Simple<char>> {
    return just('@')
        .then(text::ident())
        .then(just(' ').repeated())
        .then(just('='))
        .then(just(' ').repeated())
        .then(filter(|x: &char| x != &'\n').repeated().at_least(1))
        .padded()
        .repeated()
        .ignored()
        .map(|_| "IGNORE ME".to_string());
}