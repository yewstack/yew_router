//! Utilities for defining the parser.
use nom::IResult;
use crate::parser::RouteParserToken;
use nom::sequence::delimited;
use nom::bytes::complete::tag;
use nom::error::{context, VerboseError, ErrorKind};
use nom::multi::{many_till};
use nom::combinator::{peek};
use nom::character::complete::anychar;
use std::rc::Rc;

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



/// Similar to alt, but works on a vector of tags.
pub fn alternative(alternatives: Vec<String>) -> impl Fn(&str) -> IResult<&str, & str> {
    move |i: &str| {
        for alternative in &alternatives {
            match tag(alternative.as_str())(i) {
                done@IResult::Ok(..) => {
                    return done
                },
                _ => () // continue
            }
        }
        Err(nom::Err::Error((i, ErrorKind::Tag))) // nothing found.
    }
}


/// Consumes the input until the provided parser succeeds.
/// The consumed input is returned in the form of an allocated string.
/// # Note stop parser does not consume its input
pub fn consume_until<'a, F>(stop_parser: F) -> impl Fn(&'a str) -> IResult<&'a str, String>
where
    F: Fn(&'a str) -> IResult<&'a str, &'a str>
{
    // In order for the returned fn to be Fn instead of FnOnce, wrap the inner fn in an RC.
    let f =  Rc::new(
        many_till(
            anychar,
            peek(stop_parser), // once this succeeds, stop folding.
        )
    );
    move |i: &str| {
        let (i, (first, _stop)): (&str, (Vec<char>, &str)) = (f)(i)?;
        log::trace!("consume until - first: {:?}, stop: {}", first, _stop);
        let ret = first.into_iter().collect();
        Ok((i, ret))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn consume_until_simple() {
        let parser = consume_until(tag("z"));
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
        let parser = alternative(vec!["c", "d", "abc"].into_iter().map(String::from).collect());
        let parsed = parser("abcz").expect("Should parse");
        assert_eq!(parsed, ("z", "abc"))
    }

    #[test]
    fn alternative_and_consume_until() {
        let parser = consume_until(alternative(vec!["c", "d", "abc"].into_iter().map(String::from).collect()));
        let parsed = parser("first_stuff_abc").expect("should parse");
        assert_eq!(parsed, ("abc", "first_stuff_".to_string()))
    }
}
