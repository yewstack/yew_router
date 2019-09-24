use crate::parser::core::{capture_or_match, valid_ident_characters};
use crate::parser::RouteParserToken;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::{many0, many1};
use nom::sequence::{pair, separated_pair, tuple};
use nom::IResult;

use crate::parser::util::{optional_match, optional_matches_v, vectorize};
use nom::error::{context, VerboseError};

/// Character used to start the first query.
fn query_begin_token(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    map(char('?'), |_| RouteParserToken::QueryBegin)(i)
}

/// Character used to separate queries
fn query_separator_token(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    map(char('&'), |_| RouteParserToken::QuerySeparator)(i)
}

/// Matches
/// * "ident=item"
fn query(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    context(
        "query",
        map(
            separated_pair(valid_ident_characters, char('='), capture_or_match),
            |(ident, value)| RouteParserToken::QueryCapture {
                ident: ident.to_string(),
                capture_or_match: value,
            },
        ),
    )(i)
}

/// Matches
/// * ?ident=item
/// * ?(ident=item)
fn begin_query_parser_with_optionals(
    i: &str,
) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    context(
        "many optional queries ?()()()...",
        map(
            pair(
                query_begin_token,
                alt((vectorize(query), many1(optional_match(query)))),
            ),
            |(begin, query)| {
                let mut ret = vec![begin];
                ret.extend(query);
                ret
            },
        ),
    )(i)
}

pub(crate) fn begin_query_parser(
    i: &str,
) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    map(
        pair(query_begin_token, vectorize(query)),
        |(begin, query)| {
            let mut ret = vec![begin];
            ret.extend(query);
            ret
        },
    )(i)
}

/// Matches:
/// * &ident=item
/// * &ident=item&ident=item
/// * &ident=item&ident=item
/// * ...
fn rest_query_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    map(
        many0(tuple((query_separator_token, query))),
        |tokens: Vec<(RouteParserToken, RouteParserToken)>| {
            let new_capacity = tokens.capacity() * 2;
            tokens.into_iter().fold(
                Vec::with_capacity(new_capacity),
                |mut accumulator, element| {
                    accumulator.push(element.0);
                    accumulator.push(element.1);
                    accumulator
                },
            )
        },
    )(i)
}

fn rest_query_or_optional_rest_query(
    i: &str,
) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    alt((
        context(
            "optional & query parser",
            optional_matches_v(rest_query_parser),
        ),
        rest_query_parser,
    ))(i)
}

/// Matches:
/// * "?query=item"
/// * "?query=item&query2=item"
/// * "?query=item&query2=item&query3=item"
/// * "?query={capture}"
/// * "?query={capture}&query2=item"
/// * etc...
fn query_parser_impl(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    map(
        pair(begin_query_parser, rest_query_or_optional_rest_query),
        |(mut first, mut rest)| {
            first.append(&mut rest);
            first
        },
    )(i)
}

pub fn query_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    context(
        "query parser",
        alt((
            alt((query_parser_impl, optional_matches_v(query_parser_impl))),
            alt((
                begin_query_parser_with_optionals,
                optional_matches_v(begin_query_parser_with_optionals),
            )),
        )),
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::combinator::all_consuming;

    #[test]
    fn single_match() {
        query_parser("?lorem=ipsum").expect("should match");
    }

    #[test]
    fn single_capture() {
        query_parser("?lorem={ipsum}").expect("should match");
    }

    #[test]
    fn multiple_match() {
        query_parser("?lorem=impsum&dolor=sit").expect("should match");
    }

    #[test]
    fn multiple_capture() {
        query_parser("?lorem={ipsum}&dolor={}").expect("should match");
    }

    #[test]
    fn multiple_mixed() {
        query_parser("?lorem=ipsum&dolor={}").expect("should match");
    }

    #[test]
    fn cant_start_with_and() {
        query_parser("&query=this").expect_err("should not match");
    }

    #[test]
    fn cant_separate_with_with_question() {
        all_consuming(query_parser)("?query=this?query=that").expect_err("should not match");
    }

    #[test]
    fn optional_second_query() {
        query_parser("?query=this[&another=query]").expect("should parse");
    }

    #[test]
    fn optional_second_and_third_query() {
        query_parser("?query=this[&another=query][&yet_another=query]").expect("should parse");
    }

    #[test]
    fn optional_many_first_queries() {
        query_parser("?[query=this][another=query]").expect("should parse");
    }

    #[test]
    fn cant_have_second_query_after_optional_first_queries() {
        all_consuming(query_parser)("?[query=this][another=query]&other=thing")
            .expect_err("should not parse");
    }

    #[test]
    fn optional_query_parser() {
        query_parser("[?query=this]").expect("should parse");
    }

    #[test]
    fn optional_nested_query_parser() {
        query_parser("[?[query=this]]").expect("should parse");
    }

    #[test]
    fn optional_nested_query_parser_2() {
        query_parser("[?query=this[&otherquery=this]]").expect("should parse");
    }
}
