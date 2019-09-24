//! Utilities for defining the parser.
use crate::parser::RouteParserToken;
use nom::bytes::complete::tag;
use nom::character::complete::anychar;
use nom::character::complete::char;
use nom::combinator::{map, peek};
use nom::error::{context, ErrorKind, ParseError, VerboseError};
use nom::multi::many_till;
use nom::sequence::delimited;
use nom::{AsChar, IResult, InputIter, InputLength, InputTake, Slice};
use std::ops::RangeFrom;
use std::rc::Rc;

/// Given a function that returns a single token, wrap the token in a Vec.
pub fn vectorize<'a>(
    f: impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>> {
    move |i: &str| (f)(i).map(|(i, t)| (i, vec![t]))
}

/// Given a function that returns a vector of Tokens, optionally return a token encompassing them, should they match
pub fn optional_matches<'a, F>(
    f: F,
) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>,
{
    move |i: &str| -> IResult<&str, RouteParserToken, VerboseError<&str>> {
        let f = &f;
        context("optional matches", delimited(char('['), f, char(']')))(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(t)))
    }
}
/// Optionally match a string, returning a vector of tokens.
pub fn optional_matches_v<'a, F>(
    f: F,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>,
{
    vectorize(optional_matches(f))
}

/// Optionally match a string
pub fn optional_match<'a, F>(
    f: F,
) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>
where
    F: Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>,
{
    move |i: &str| -> IResult<&str, RouteParserToken, VerboseError<&str>> {
        let f = &f;
        context("optional match", delimited(char('['), f, char(']')))(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(vec![t])))
    }
}

/// Similar to alt, but works on a vector of tags.
pub fn alternative(alternatives: Vec<String>) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| {
        for alternative in &alternatives {
            if let done @ IResult::Ok(..) = tag(alternative.as_str())(i) {
                return done;
            }
        }
        Err(nom::Err::Error((i, ErrorKind::Tag))) // nothing found.
    }
}

/// Consumes the input until the provided parser succeeds.
/// The consumed input is returned in the form of an allocated string.
/// # Note
/// `stop_parser` only peeks its input.
pub fn consume_until<'a, F, E>(stop_parser: F) -> impl Fn(&'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str>,
    F: Fn(&'a str) -> IResult<&'a str, &'a str, E>,
{
    // In order for the returned fn to be Fn instead of FnOnce, wrap the inner fn in an RC.
    let f = Rc::new(many_till(
        anychar,
        peek(stop_parser), // once this succeeds, stop folding.
    ));
    move |i: &str| {
        let (i, (first, _stop)): (&str, (Vec<char>, &str)) = (f)(i)?;
        let ret = first.into_iter().collect();
        Ok((i, ret))
    }
}

// TODO This might be the same as preceeded.
/// Skips values until the stop parser succeeds, then returns the output of the stop parser.
pub fn skip_until<I, F, E, T>(stop_parser: F) -> impl Fn(I) -> IResult<I, T, E>
where
    I: Clone + PartialEq + InputIter + InputLength + InputTake + Slice<RangeFrom<usize>>,
    <I as InputIter>::Item: AsChar,
    E: ParseError<I>,
    F: Fn(I) -> IResult<I, T, E>,
{
    map(
        many_till(anychar, stop_parser),
        |(_any, stop_parser_result)| stop_parser_result,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn consume_until_simple() {
        let parser = consume_until::<_, ()>(tag("z"));
        let parsed = parser("abcz").expect("Should parse");
        assert_eq!(parsed, ("z", "abc".to_string()))
    }

    #[test]
    fn consume_until_fail() {
        let parser = consume_until(tag("z"));
        let e = parser("abc").expect_err("Should parse");
        assert_eq!(e, nom::Err::Error(("", ErrorKind::Eof)))
    }

    #[test]
    fn alternative_simple() {
        let parser = alternative(
            vec!["c", "d", "abc"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let parsed = parser("abcz").expect("Should parse");
        assert_eq!(parsed, ("z", "abc"))
    }

    #[test]
    fn alternative_and_consume_until() {
        let parser = consume_until(alternative(
            vec!["c", "d", "abc"]
                .into_iter()
                .map(String::from)
                .collect(),
        ));
        let parsed = parser("first_stuff_abc").expect("should parse");
        assert_eq!(parsed, ("abc", "first_stuff_".to_string()))
    }

    #[test]
    fn simple_skip_until() {
        let parsed =
            skip_until::<_, _, (), _>(tag("done"))("useless_stuff_done").expect("should parse");
        assert_eq!(parsed, ("", "done"))
    }
}
