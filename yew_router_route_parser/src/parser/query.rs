use nom::IResult;
use crate::parser::Token;
use nom::sequence::{tuple, separated_pair};
use nom::multi::many0;
use nom::combinator::map;
use nom::bytes::complete::tag;
use crate::parser::core::{capture_or_match, valid_ident_characters};


fn query_begin_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("?"),
        |_| Token::QueryBegin
    )(i)
}
fn query_separator_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("&"),
        |_| Token::QuerySeparator
    )(i)
}


/// matches "item=item" and "item={capture}"
fn query(i: &str) -> IResult<&str, Token> {
    map(
        separated_pair(valid_ident_characters, tag("=",), capture_or_match),
        |(ident, value)| Token::QueryCapture {ident: ident.to_string(), capture_or_match: value }
    )(i)
}

/// Matches:
/// * "?query=item"
/// * "?query=item&query2=item"
/// * "?query=item&query2=item&query3=item"
/// * "?query={capture}"
/// * "?query={capture}&query2=item"
/// * etc...
pub fn query_parser(i: &str) -> IResult<&str, Vec<Token>> {
    fn begin_query_parser(i: &str) -> IResult<&str, (Token, Token)> {
        tuple(
            (
                query_begin_token,
                query
            )
        )(i)
    }

    fn rest_query_parser(i: &str) -> IResult<&str, Vec<Token>> {
        map(
            many0(tuple(
                (
                    query_separator_token,
                    query
                )
            )),
            |tokens: Vec<(Token, Token)>| {
                let new_capacity = tokens.capacity() * 2;
                tokens.into_iter().fold(Vec::with_capacity(new_capacity), |mut accumulator, element| {
                    accumulator.push(element.0);
                    accumulator.push(element.1);
                    accumulator
                })

            }
        )(i)
    }

    map(
        tuple(
            (
                begin_query_parser,
                rest_query_parser
            )
        ),
        |(first, mut rest)| {
            let mut tokens = vec![first.0, first.1];
            tokens.append(&mut rest);
            tokens
        }
    )(i)
}