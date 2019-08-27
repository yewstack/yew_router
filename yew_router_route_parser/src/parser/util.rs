//! Utilities for defining the parser.
use nom::IResult;
use crate::parser::RouteParserToken;
use nom::sequence::delimited;
use nom::bytes::complete::tag;

pub fn ret_vec<'a>(f: impl Fn(&'a str) -> IResult<&'a str, RouteParserToken>) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>> {
    move |i: &str | {
        (f)(i).map(|(i, t)| (i, vec![t]))
    }
}

pub fn optional_matches<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>>
{
    move |i: &str| -> IResult<&str, RouteParserToken> {
        let f = &f;
        delimited(tag("("), f, tag(")"))(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(t)))
    }
}

pub fn optional_matches_v<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>>
{
    ret_vec(optional_matches(f))
}

#[allow(dead_code)]
pub fn optional_match<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken>
    where
        F: Fn(&'a str) -> IResult<&'a str, RouteParserToken>
{
    move |i: &str| -> IResult<&str, RouteParserToken> {
        let f = &f;
        delimited(tag("("), f, tag(")"))(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(vec![t])))
    }
}
