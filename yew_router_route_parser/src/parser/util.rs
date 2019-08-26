use nom::IResult;
use crate::parser::Token;
use nom::sequence::delimited;
use nom::bytes::complete::tag;

pub fn ret_vec<'a, >(f: impl Fn(&'a str) -> IResult<&'a str, Token>) -> impl Fn(&'a str) -> IResult<&'a str, Vec<Token>> {
    move |i: &str | {
        (f)(i).map(|(i, t)| (i, vec![t]))
    }
}

pub fn optional_matches<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, Token>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<Token>>
{
    move |i: &str| -> IResult<&str, Token> {
        let f = &f;
        delimited(tag("("), f, tag(")"))(i)
            .map(|(i, t)| (i, Token::Optional(t)))
    }
}

pub fn optional_matches_v<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, Vec<Token>>
    where
        F: Fn(&'a str) -> IResult<&'a str, Vec<Token>>
{
    ret_vec(optional_matches(f))
}

pub fn optional_match<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, Token>
    where
        F: Fn(&'a str) -> IResult<&'a str, Token>
{
    move |i: &str| -> IResult<&str, Token> {
        let f = &f;
        delimited(tag("("), f, tag(")"))(i)
            .map(|(i, t)| (i, Token::Optional(vec![t])))
    }
}
