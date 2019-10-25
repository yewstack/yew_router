use crate::{
    error::{ExpectedToken, ParserErrorReason},
    parser::{CaptureOrExact, RefCaptureVariant, RouteParserToken},
    ParseError,
};
use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::{
        complete::{char, digit1},
        is_digit,
    },
    combinator::{map, map_parser},
    error::ErrorKind,
    sequence::{delimited, separated_pair},
    IResult,
};

/// Indicates if the parser is working to create a matcher for a datastructure with named or unnamed fields.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum FieldType {
    /// For Thing { field: String }
    Named,
    /// for Thing(String)
    Unnamed,
}

pub fn get_slash(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('/'), |_: char| RouteParserToken::Separator)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Separator)))
}

pub fn get_question(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('?'), |_: char| RouteParserToken::QueryBegin)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::QueryBegin)))
}

pub fn get_and(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('&'), |_: char| RouteParserToken::QuerySeparator)(i).map_err(|_: nom::Err<()>| {
        nom::Err::Error(ParseError::expected(ExpectedToken::QuerySeparator))
    })
}

/// Returns a FragmentBegin variant if the next character is '\#'.
pub fn get_hash(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('#'), |_: char| RouteParserToken::FragmentBegin)(i).map_err(|_: nom::Err<()>| {
        nom::Err::Error(ParseError::expected(ExpectedToken::FragmentBegin))
    })
}

/// Returns an End variant if the next character is a '!`.
pub fn get_end(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(char('!'), |_: char| RouteParserToken::End)(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::End)))
}

/// Returns an End variant if the next character is a '!`.
fn get_open_bracket(i: &str) -> IResult<&str, (), ParseError> {
    map(char('{'), |_: char| ())(i).map_err(|_: nom::Err<()>| {
        nom::Err::Error(ParseError::expected(ExpectedToken::OpenBracket))
    })
}

fn get_close_bracket(i: &str) -> IResult<&str, (), ParseError> {
    map(char('}'), |_: char| ())(i).map_err(|_: nom::Err<()>| {
        nom::Err::Error(ParseError::expected(ExpectedToken::CloseBracket))
    })
}

fn get_eq(i: &str) -> IResult<&str, (), ParseError> {
    map(char('='), |_: char| ())(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Equals)))
}

fn get_star(i: &str) -> IResult<&str, (), ParseError> {
    map(char('*'), |_: char| ())(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Star)))
}

fn get_colon(i: &str) -> IResult<&str, (), ParseError> {
    map(char(':'), |_: char| ())(i)
        .map_err(|_: nom::Err<()>| nom::Err::Error(ParseError::expected(ExpectedToken::Colon)))
}

fn rust_ident(i: &str) -> IResult<&str, &str, ParseError> {
    let invalid_ident_chars = r##" \|/{[]()?+=-!@#$%^&*~`'";:"##;
    map_parser(take_till1(move |c| c == '}'), move |i: &str| {
        match take_till1::<_, _, ()>(|c| invalid_ident_chars.contains(c))(i) {
            Ok((remain, got)) => {
                if got.len() > 0 && got.starts_with(|c: char| is_digit(c as u8)) {
                    Err(nom::Err::Failure(ParseError {
                        reason: Some(ParserErrorReason::BadRustIdent(got.chars().next().unwrap())),
                        expected: vec![ExpectedToken::Ident],
                        offset: 1,
                    }))
                } else if remain.len() > 0 {
                    Err(nom::Err::Failure(ParseError {
                        reason: Some(ParserErrorReason::BadRustIdent(
                            remain.chars().next().unwrap(),
                        )),
                        expected: vec![ExpectedToken::CloseBracket, ExpectedToken::Ident],
                        offset: got.len() + 1,
                    }))
                } else {
                    Ok((i, i))
                }
            }
            Err(_) => {
                Ok((i, i)) // TODO this might be right?
            }
        }
    })(i)
}

fn exact_impl(i: &str) -> IResult<&str, &str, ParseError> {
    let special_chars = r##"/?&#={}!"##; // TODO these might allow escaping one day.
    take_till1(move |c| special_chars.contains(c))(i).map_err(|x: nom::Err<(&str, ErrorKind)>| {
        let s = match x {
            nom::Err::Error((s, _)) => s,
            nom::Err::Failure((s, _)) => s,
            nom::Err::Incomplete(_) => panic!(),
        };
        nom::Err::Error(ParseError {
            reason: Some(ParserErrorReason::BadLiteral),
            expected: vec![ExpectedToken::Literal],
            offset: 1 + i.len() - s.len(),
        })
    })
}

pub fn exact(i: &str) -> IResult<&str, RouteParserToken, ParseError> {
    map(exact_impl, RouteParserToken::Exact)(i)
}

pub fn capture<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken<'a>, ParseError> {
    map(capture_impl(field_type), RouteParserToken::Capture)
}

pub fn capture_single<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken<'a>, ParseError> {
    map(capture_single_impl(field_type), RouteParserToken::Capture)
}

fn capture_single_impl<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, RefCaptureVariant<'a>, ParseError> {
    move |i: &str| match field_type {
        FieldType::Named => delimited(
            get_open_bracket,
            named::single_capture_impl,
            get_close_bracket,
        )(i),
        FieldType::Unnamed => delimited(
            get_open_bracket,
            alt((named::single_capture_impl, unnamed::single_capture_impl)),
            get_close_bracket,
        )(i),
    }
}

/// Captures {ident}, {*:ident}, {<number>:ident}
fn capture_impl<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, RefCaptureVariant, ParseError> {
    move |i: &str| match field_type {
        FieldType::Named => {
            let inner = alt((
                named::many_capture_impl,
                named::numbered_capture_impl,
                named::single_capture_impl,
            ));
            delimited(get_open_bracket, inner, get_close_bracket)(i)
        }
        FieldType::Unnamed => {
            let inner = alt((
                named::many_capture_impl,
                unnamed::many_capture_impl,
                named::numbered_capture_impl,
                unnamed::numbered_capture_impl,
                named::single_capture_impl,
                unnamed::single_capture_impl,
            ));
            delimited(get_open_bracket, inner, get_close_bracket)(i)
        }
    }
}

mod named {
    use super::*;
    pub fn single_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        map(rust_ident, |key| RefCaptureVariant::Named(key))(i)
    }

    pub fn many_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        map(
            separated_pair(get_star, get_colon, rust_ident),
            |(_, key)| RefCaptureVariant::ManyNamed(key),
        )(i)
    }

    pub fn numbered_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        map(
            separated_pair(digit1, get_colon, rust_ident),
            |(number, key)| RefCaptureVariant::NumberedNamed {
                sections: number.parse().unwrap(),
                name: key,
            },
        )(i)
    }
}

mod unnamed {
    use super::*;

    /// #Note
    /// because this always succeeds, try this last
    pub fn single_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        Ok((i, RefCaptureVariant::Unnamed))
    }

    pub fn many_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        map(get_star, |_| RefCaptureVariant::ManyUnnamed)(i)
    }

    pub fn numbered_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant, ParseError> {
        map(digit1, |number: &str| RefCaptureVariant::NumberedUnnamed {
            sections: number.parse().unwrap(),
        })(i)
    }
}

/// Gets a capture or exact, mapping it to the CaptureOrExact enum - to provide a limited subset.
fn cap_or_exact<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, CaptureOrExact<'a>, ParseError> {
    move |i: &str| {
        alt((
            map(capture_single_impl(field_type), CaptureOrExact::Capture),
            map(exact_impl, CaptureOrExact::Exact),
        ))(i)
    }
}

/// Matches a query
pub fn query<'a>(
    field_type: FieldType,
) -> impl Fn(&'a str) -> IResult<&'a str, RouteParserToken<'a>, ParseError> {
    move |i: &str| {
        map(
            separated_pair(exact_impl, get_eq, cap_or_exact(field_type)),
            |(ident, capture_or_exact)| RouteParserToken::Query {
                ident,
                capture_or_exact,
            },
        )(i)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cap_or_exact_match_lit() {
        cap_or_exact(FieldType::Named)("lorem").expect("Should parse");
    }
    #[test]
    fn cap_or_exact_match_cap() {
        cap_or_exact(FieldType::Named)("{lorem}").expect("Should parse");
    }

    #[test]
    fn query_section_exact() {
        query(FieldType::Named)("lorem=ipsum").expect("should parse");
    }

    #[test]
    fn query_section_capture_named() {
        query(FieldType::Named)("lorem={ipsum}").expect("should parse");
    }
    #[test]
    fn query_section_capture_named_fails_without_key() {
        query(FieldType::Named)("lorem={}").expect_err("should not parse");
    }
    #[test]
    fn query_section_capture_unnamed_succeeds_without_key() {
        query(FieldType::Unnamed)("lorem={}").expect("should parse");
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
