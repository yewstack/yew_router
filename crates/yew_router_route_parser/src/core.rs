use nom::IResult;
use crate::parser::{RouteParserToken, RefCaptureVariant, CaptureOrExact};
use nom::sequence::{separated_pair, delimited, pair};
use nom::combinator::{map, map_opt, map_parser};
use nom::branch::alt;
use nom::character::complete::digit1;
use crate::error::{ParserErrorReason, ExpectedToken};
use nom::bytes::complete::{take_until, tag, take_till1};
use nom::character::complete::char;
use crate::ParseError;
use nom::error::ErrorKind;
use nom::character::is_digit;

pub fn get_slash(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('/'), |_: char| RouteParserToken::Separator)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Separator)))
}

pub fn get_question(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('?'), |_: char| RouteParserToken::QueryBegin)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::QueryBegin)))
}

pub fn get_and(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('&'), |_: char| RouteParserToken::QuerySeparator)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::QuerySeparator)))
}

/// Returns a FragmentBegin variant if the next character is '\#'.
pub fn get_hash(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('#'), |_: char| RouteParserToken::FragmentBegin)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::FragmentBegin)))
}

/// Returns an End variant if the next character is a '!`.
pub fn get_end(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('!'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::End)))
}

/// Returns an End variant if the next character is a '!`.
fn get_open_bracket(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('{'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::OpenBracket)))
}

fn get_close_bracket(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('}'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::CloseBracket)))
}

fn get_eq(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('='), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Equals)))
}

fn get_star(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('*'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Star)))
}

fn get_colon(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char(':'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Colon)))
}

pub fn rust_ident(i: &str) -> IResult<&str, &str, ParseError> {

    let invalid_ident_chars = r##" \|/{[]()?+=-!@#$%^&*~`'";:"##;
    map_parser(
        take_till1(move |c| {
            c == '}'
        }),
        move |i: &str| {
            match take_till1::<_,_,()>(|c| invalid_ident_chars.contains(c))(i) {
                Ok((remain, got)) => {
                    if got.len() > 0 && got.starts_with(|c: char|is_digit(c as u8)) {
                        Err(nom::Err::Failure(ParseError {
                            reason: Some(ParserErrorReason::BadRustIdent(got.chars().next().unwrap())),
                            expected: vec![ExpectedToken::Ident],
                            offset: 1
                        }))
                    }
                    else if remain.len() > 0 {
                        Err(nom::Err::Failure(ParseError {
                            reason: Some(ParserErrorReason::BadRustIdent(remain.chars().next().unwrap())),
                            expected: vec![ExpectedToken::CloseBracket, ExpectedToken::Ident],
                            offset: got.len() + 1
                        }))
                    } else {
                        Ok((i,i))
                    }
                },
                Err(_) => {
                   Ok((i,i)) // TODO this might be right?
                }
            }
        }
    )(i)
}

pub fn exact_impl(i: &str) -> IResult<&str, &str, ParseError> {
    let special_chars = r##"/?&#={}!"##; // TODO these might allow escaping one day.
    take_till1(move |c| special_chars.contains(c))(i)
        .map_err(|x: nom::Err<(&str, ErrorKind)>| {
            let s = match x {
                nom::Err::Error((s,_)) => s,
                nom::Err::Failure((s,_)) => s,
                nom::Err::Incomplete(_) => panic!(),
            };
            nom::Err::Error(ParseError{
                reason: Some(ParserErrorReason::BadLiteral),
                expected: vec![ExpectedToken::Literal],
                offset: 1 + i.len() - s.len()
            })
        })
}

pub fn exact(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(exact_impl, |s| RouteParserToken::Exact(s))(i)
}


pub fn capture(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(named_capture_impl, |cv: RefCaptureVariant| {
        RouteParserToken::Capture(cv)
    })(i)
}

pub fn capture_single(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(
        delimited(get_open_bracket, single_capture_impl, get_close_bracket),
        RouteParserToken::Capture,
    )(i)
}

/// Captures {ident}, {*:ident}, {<number>:ident}
pub fn named_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
    let inner = alt((
        many_capture_impl,
        numbered_capture_impl,
        single_capture_impl,
    ));
    delimited(get_open_bracket, inner, get_close_bracket)(i)
}

fn single_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
    map(rust_ident, |key| RefCaptureVariant::Named(key))(i)
}

fn many_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
    map(
        separated_pair(get_star, get_colon, rust_ident),
        |(_, key)| RefCaptureVariant::ManyNamed(key),
    )(i)
}

fn numbered_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
    map(
        separated_pair(digit1, get_colon, rust_ident),
        |(number, key)| RefCaptureVariant::NumberedNamed {
            sections: number.parse().unwrap(),
            name: key,
        },
    )(i)
}

/// Gets a capture or exact, mapping it to the CaptureOrExact enum - to provide a limited subset.
pub fn cap_or_exact(i: &str) -> IResult<&str, CaptureOrExact, ParseError> {
    alt((
        map(
            delimited(get_open_bracket, single_capture_impl, get_close_bracket),
            CaptureOrExact::Capture,
        ),
        map(exact_impl, |exact| CaptureOrExact::Exact(exact)),
    ))(i)
}

/// Matches a query
pub fn query(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(
        separated_pair(exact_impl, get_eq, cap_or_exact),
        |(ident, capture_or_exact)| RouteParserToken::Query {
            ident,
            capture_or_exact,
        },
    )(i)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cap_or_exact_match_lit() {
        cap_or_exact("lorem").expect("Should parse");
    }
    #[test]
    fn cap_or_exact_match_cap() {
        cap_or_exact("{lorem}").expect("Should parse");
    }
    #[test]
    fn query_section() {
        query("lorem=ipsum").expect("should parse");
    }

    #[test]
    fn non_leading_numbers_in_ident() {
        rust_ident("hello5").expect("sholud parse");
    }
    #[test]
    fn leading_numbers_in_ident_fails() {
        rust_ident("5hello").expect_err("sholud not parse");
    }
}
