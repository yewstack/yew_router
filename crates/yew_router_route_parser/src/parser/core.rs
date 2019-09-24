//! Core functions for working with the route parser.
use crate::parser::CaptureOrExact;
use crate::parser::CaptureVariant;
use crate::parser::RouteParserToken;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take};
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::is_digit;
use nom::combinator::{map, peek};
use nom::error::ParseError;
use nom::error::{context, ErrorKind, VerboseError};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::IResult;

/// Captures a string up to the point where a character not possible to be present in Rust's identifier is encountered.
/// It prevents the first character from being a digit.
pub fn valid_ident_characters(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    const INVALID_CHARACTERS: &str = " -*/+#?&^@%$\'\"`%~;,.|\\{}[]()<>=\t\n";
    context("valid ident", |i: &str| {
        let (i, next) = peek(take(1usize))(i)?; // Look at the first character
        if is_digit(next.bytes().next().unwrap()) {
            Err(nom::Err::Error(VerboseError::from_error_kind(
                i,
                ErrorKind::Digit,
            )))
        } else {
            is_not(INVALID_CHARACTERS)(i)
        }
    })(i)
}

/// A more permissive set of characters than those specified in `valid_ident_characters that the route string will need to match exactly.
pub fn valid_exact_match_characters(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    const INVALID_CHARACTERS: &str = " /?&#=\t\n(){}";
    context("valid exact match", |i: &str| is_not(INVALID_CHARACTERS)(i))(i)
}

/// Captures groups of characters that will need to be matched exactly later.
pub fn match_exact(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    context(
        "match",
        map(valid_exact_match_characters, |ident| {
            RouteParserToken::Exact(ident.to_string())
        }),
    )(i)
}

/// Matches any of the capture variants
///
/// * {}
/// * {*}
/// * {5}
/// * {name}
/// * {*:name}
/// * {5:name}
pub fn capture(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    let capture_variants = alt((
        context(
            "capture unnamed",
            map(peek(char('}')), |_| CaptureVariant::Unnamed),
        ), // just empty {}
        context(
            "capture many named",
            map(preceded(tag("*:"), valid_ident_characters), |s| {
                CaptureVariant::ManyNamed(s.to_string())
            }),
        ),
        context(
            "capture many unnamed",
            map(char('*'), |_| CaptureVariant::ManyUnnamed),
        ),
        context(
            "capture named",
            map(valid_ident_characters, |s| {
                CaptureVariant::Named(s.to_string())
            }),
        ),
        context(
            "capture number named",
            map(
                separated_pair(digit1, char(':'), valid_ident_characters),
                |(n, s)| CaptureVariant::NumberedNamed {
                    sections: n.parse().expect("Should parse digits"),
                    name: s.to_string(),
                },
            ),
        ),
        context(
            "capture number unnamed",
            map(digit1, |num: &str| CaptureVariant::NumberedUnnamed {
                sections: num.parse().expect("should parse digits"),
            }),
        ),
    ));

    context(
        "capture",
        map(
            delimited(char('{'), capture_variants, char('}')),
            RouteParserToken::Capture,
        ),
    )(i)
}

/// Matches either "item" or "{capture}"
/// It returns a subset enum of Token.
pub fn capture_or_match(i: &str) -> IResult<&str, CaptureOrExact, VerboseError<&str>> {
    let (i, token) = context("capture or match", alt((capture, match_exact)))(i)?;
    let token = match token {
        RouteParserToken::Capture(variant) => CaptureOrExact::Capture(variant),
        RouteParserToken::Exact(m) => CaptureOrExact::Exact(m),
        _ => unreachable!("Only should handle captures and matches"),
    };
    Ok((i, token))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn match_any() {
        let cap = capture("{}").expect("Should match").1;
        assert_eq!(cap, RouteParserToken::Capture(CaptureVariant::Unnamed));
    }

    #[test]
    fn capture_named_test() {
        let cap = capture("{loremipsum}").unwrap();
        assert_eq!(
            cap,
            (
                "",
                RouteParserToken::Capture(CaptureVariant::Named("loremipsum".to_string()))
            )
        );
    }

    #[test]
    fn capture_many_unnamed_test() {
        let cap = capture("{*}").unwrap();
        assert_eq!(
            cap,
            ("", RouteParserToken::Capture(CaptureVariant::ManyUnnamed))
        );
    }

    #[test]
    fn capture_unnamed_test() {
        let cap = capture("{}").unwrap();
        assert_eq!(
            cap,
            ("", RouteParserToken::Capture(CaptureVariant::Unnamed))
        );
    }

    #[test]
    fn capture_numbered_unnamed_test() {
        let cap = capture("{5}").unwrap();
        assert_eq!(
            cap,
            (
                "",
                RouteParserToken::Capture(CaptureVariant::NumberedUnnamed { sections: 5 })
            )
        );
    }

    #[test]
    fn capture_numbered_named_test() {
        let cap = capture("{5:name}").unwrap();
        assert_eq!(
            cap,
            (
                "",
                RouteParserToken::Capture(CaptureVariant::NumberedNamed {
                    sections: 5,
                    name: "name".to_string()
                })
            )
        );
    }

    #[test]
    fn capture_many_named() {
        let cap = capture("{*:name}").unwrap();
        assert_eq!(
            cap,
            (
                "",
                RouteParserToken::Capture(CaptureVariant::ManyNamed("name".to_string()))
            )
        );
    }

    #[test]
    fn rejects_invalid_ident() {
        valid_ident_characters("+-lorem").expect_err("Should reject at +");
    }

    #[test]
    fn accepts_valid_ident() {
        valid_ident_characters("Lorem").expect("Should accept");
    }

    #[test]
    fn capture_consumes() {
        capture("{aoeu").expect_err("Should not complete");
    }

}
