//! Utilities for defining the parser.
use nom::IResult;
use crate::parser::RouteParserToken;
use nom::sequence::delimited;
use nom::bytes::complete::tag;
use nom::error::{context, VerboseError};

pub fn ret_vec<'a>(f: impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>> {
    move |i: &str | {
        (f)(i).map(|(i, t)| (i, vec![t]))
    }
}

pub fn optional_matches<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>
{
    move |i: &str| -> IResult<&str, RouteParserToken, VerboseError<&str>> {
        let f = &f;
        context(
            "optional matches",
            delimited(tag("("), f, tag(")"))
        )(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(t)))
    }
}

pub fn optional_matches_v<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<RouteParserToken>, VerboseError<&'a str>>
{
    ret_vec(optional_matches(f))
}


pub fn optional_match<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>
    where
        F: Fn(&'a str) -> IResult<&'a str, RouteParserToken, VerboseError<&'a str>>
{
    move |i: &str| -> IResult<&str, RouteParserToken, VerboseError<&str>> {
        let f = &f;
        context(
            "optional match",
            delimited(tag("("), f, tag(")"))
        )(i)
            .map(|(i, t)| (i, RouteParserToken::Optional(vec![t])))
    }
}
