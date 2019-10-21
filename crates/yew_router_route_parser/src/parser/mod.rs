//! Parser that consumes a string and produces the first representation of the matcher.
use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::complete::{char, digit1},
    combinator::map,
    sequence::{delimited, separated_pair},
    IResult,
};

mod error;
mod optimizer;
use crate::parser::error::{ExpectedToken, ParserErrorReason};
pub use error::{ParseError2, PrettyParseError};
pub use optimizer::{convert_tokens, parse_str_and_optimize_tokens};

/// Tokens generated from parsing a route matcher string.
/// They will be optimized to another token type that is used to match URLs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteParserToken<'a> {
    /// Match /
    Separator,
    /// Match a specific string.
    Exact(&'a str),
    /// Match {_}. See `CaptureVariant` for more.
    Capture(RefCaptureVariant<'a>),
    /// Match ?
    QueryBegin,
    /// Match &
    QuerySeparator,
    /// Match x=y
    QueryCapture {
        /// Identifier
        ident: &'a str,
        /// Capture or match
        capture_or_match: CaptureOrExact<'a>,
    },
    /// Match \#
    FragmentBegin,
    /// Match !
    End,
}

/// Token representing various types of captures.
///
/// It can capture and discard for unnamed variants, or capture and store in the `Matches` for the
/// named variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefCaptureVariant<'a> {
    /// {name} - captures a section and adds it to the map with a given name.
    Named(&'a str),
    /// {*:name} - captures over many sections and adds it to the map with a given name.
    ManyNamed(&'a str),
    /// {2:name} - captures a fixed number of sections with a given name.
    NumberedNamed {
        /// Number of sections to match.
        sections: usize,
        /// The key to be entered in the `Matches` map.
        name: &'a str,
    },
}

/// Either a Capture, or an Exact match
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureOrExact<'a> {
    /// Match a specific string.
    Exact(&'a str),
    /// Match a capture variant.
    Capture(RefCaptureVariant<'a>),
}


/// Represents the states the parser can be in.
#[derive(Clone, PartialEq)]
enum ParserState<'a> {
    None,
    Path { prev_token: RouteParserToken<'a> },
    FirstQuery { prev_token: RouteParserToken<'a> },
    NthQuery { prev_token: RouteParserToken<'a> },
    Fragment { prev_token: RouteParserToken<'a> },
    End,
}
impl<'a> ParserState<'a> {
    /// Given a new route parser token, transition to a new state.
    ///
    /// This will set the prev token to a token able to be handled by the new state,
    /// so the new state does not need to handle arbitrary "from" states.
    ///
    /// This function represents the valid state transition graph.
    fn transition(self, token: RouteParserToken<'a>) -> Result<Self, ParserErrorReason> {
        match self {
            ParserState::None => match token {
                RouteParserToken::Separator => Ok(ParserState::Path { prev_token: token }),
                RouteParserToken::Exact(_) => Err(ParserErrorReason::NotAllowedStateTransition),
                RouteParserToken::Capture(_) => Ok(ParserState::Path { prev_token: token }), /* TODO revise decision to allow this state transform for _all_ capture variants. */
                RouteParserToken::QueryBegin => Ok(ParserState::FirstQuery { prev_token: token }),
                RouteParserToken::QuerySeparator => {
                    Err(ParserErrorReason::NotAllowedStateTransition)
                }
                RouteParserToken::QueryCapture { .. } => {
                    Err(ParserErrorReason::NotAllowedStateTransition)
                }
                RouteParserToken::FragmentBegin => Ok(ParserState::Fragment { prev_token: token }),
                RouteParserToken::End => Err(ParserErrorReason::NotAllowedStateTransition),
            },
            ParserState::Path { prev_token } => {
                match prev_token {
                    RouteParserToken::Separator => match token {
                        RouteParserToken::Exact(_) | RouteParserToken::Capture(_) => {
                            Ok(ParserState::Path { prev_token: token })
                        }
                        RouteParserToken::QueryBegin => {
                            Ok(ParserState::FirstQuery { prev_token: token })
                        }
                        RouteParserToken::FragmentBegin => {
                            Ok(ParserState::Fragment { prev_token: token })
                        }
                        RouteParserToken::End => Ok(ParserState::End),
                        _ => Err(ParserErrorReason::NotAllowedStateTransition),
                    },
                    RouteParserToken::Exact(_) => match token {
                        RouteParserToken::Separator | RouteParserToken::Capture(_) => {
                            Ok(ParserState::Path { prev_token: token })
                        }
                        RouteParserToken::QueryBegin => {
                            Ok(ParserState::FirstQuery { prev_token: token })
                        }
                        RouteParserToken::FragmentBegin => {
                            Ok(ParserState::Fragment { prev_token: token })
                        }
                        RouteParserToken::End => Ok(ParserState::End),
                        _ => Err(ParserErrorReason::NotAllowedStateTransition),
                    },
                    RouteParserToken::Capture(_) => match token {
                        RouteParserToken::Separator | RouteParserToken::Exact(_) => {
                            Ok(ParserState::Path { prev_token: token })
                        }
                        RouteParserToken::QueryBegin => {
                            Ok(ParserState::FirstQuery { prev_token: token })
                        }
                        RouteParserToken::FragmentBegin => {
                            Ok(ParserState::Fragment { prev_token: token })
                        }
                        RouteParserToken::End => Ok(ParserState::End),
                        _ => Err(ParserErrorReason::NotAllowedStateTransition),
                    },
                    _ => Err(ParserErrorReason::InvalidState), /* Other previous token types are
                                                                * invalid within a Path state. */
                }
            }
            ParserState::FirstQuery { prev_token } => match prev_token {
                RouteParserToken::QueryBegin => match token {
                    RouteParserToken::QueryCapture { .. } => {
                        Ok(ParserState::FirstQuery { prev_token: token })
                    }
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                RouteParserToken::QueryCapture { .. } => match token {
                    RouteParserToken::QuerySeparator => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    RouteParserToken::FragmentBegin => {
                        Ok(ParserState::Fragment { prev_token: token })
                    }
                    RouteParserToken::End => Ok(ParserState::End),
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                _ => Err(ParserErrorReason::InvalidState),
            },
            ParserState::NthQuery { prev_token } => match prev_token {
                RouteParserToken::QuerySeparator => match token {
                    RouteParserToken::QueryCapture { .. } => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                RouteParserToken::QueryCapture { .. } => match token {
                    RouteParserToken::QuerySeparator => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    RouteParserToken::FragmentBegin => {
                        Ok(ParserState::Fragment { prev_token: token })
                    }
                    RouteParserToken::End => Ok(ParserState::End),
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                _ => Err(ParserErrorReason::InvalidState),
            },
            ParserState::Fragment { prev_token } => match prev_token {
                RouteParserToken::FragmentBegin
                | RouteParserToken::Exact(_)
                | RouteParserToken::Capture(_) => Ok(ParserState::Fragment { prev_token: token }),
                RouteParserToken::End => Ok(ParserState::End),
                _ => Err(ParserErrorReason::InvalidState),
            },
            ParserState::End => Err(ParserErrorReason::TokensAfterEndToken),
        }
    }
}


/// Parse a matching string into a vector of RouteParserTokens.
pub fn parse(mut i: &str) -> Result<Vec<RouteParserToken>, PrettyParseError> {
    let input = i;
    let mut tokens: Vec<RouteParserToken> = vec![];
    let mut state = ParserState::None;

    loop {
        let (ii, token) = parse_impl(i, &state).map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => PrettyParseError {
                error: e,
                input,
                remaining: i,
            },
            _ => panic!("parser should not be incomplete"),
        })?;
        i = ii;
        state = state.transition(token.clone()).map_err(|reason| {
            let error = ParseError2 {
                reason: Some(reason),
                expected: vec![],
            };
            PrettyParseError {
                error,
                input,
                remaining: i,
            }
        })?;
        tokens.push(token);

        // If there is no more input, break out of the loop
        if i.is_empty() {
            break;
        }
    }
    Ok(tokens)
}

fn parse_impl<'a>(
    i: &'a str,
    state: &ParserState,
) -> IResult<&'a str, RouteParserToken<'a>, ParseError2> {
    match state {
        ParserState::None => alt((get_slash, get_question, get_hash, capture))(i)
            .map_err(|_| ParseError2 {
                reason: None,
                expected: vec![
                    ExpectedToken::Separator,
                    ExpectedToken::QueryBegin,
                    ExpectedToken::FragmentBegin,
                    ExpectedToken::CaptureNamed,
                ],
            })
            .map_err(nom::Err::Error),
        ParserState::Path { prev_token } => match prev_token {
            RouteParserToken::Separator => {
                alt((exact, capture, get_question, get_hash, get_end))(i)
                    .map_err(|_| ParseError2 {
                        reason: None,
                        expected: vec![
                            ExpectedToken::Literal,
                            ExpectedToken::CaptureNamed,
                            ExpectedToken::CaptureManyNamed,
                            ExpectedToken::CaptureNumberedNamed,
                            ExpectedToken::QueryBegin,
                            ExpectedToken::FragmentBegin,
                            ExpectedToken::End,
                        ],
                    })
                    .map_err(nom::Err::Error)
            }
            RouteParserToken::Exact(_) => {
                alt((get_slash, capture, get_question, get_hash, get_end))(i)
                    .map_err(|_| ParseError2 {
                        reason: None,
                        expected: vec![
                            ExpectedToken::Separator,
                            ExpectedToken::CaptureNamed,
                            ExpectedToken::CaptureManyNamed,
                            ExpectedToken::CaptureNumberedNamed,
                            ExpectedToken::QueryBegin,
                            ExpectedToken::FragmentBegin,
                            ExpectedToken::End,
                        ],
                    })
                    .map_err(nom::Err::Error)
            }
            RouteParserToken::Capture(_) => {
                alt((get_slash, exact, get_question, get_hash, get_end))(i)
                    .map_err(|_| ParseError2 {
                        reason: None,
                        expected: vec![
                            ExpectedToken::Separator,
                            ExpectedToken::CaptureNamed,
                            ExpectedToken::CaptureManyNamed,
                            ExpectedToken::CaptureNumberedNamed,
                            ExpectedToken::QueryBegin,
                            ExpectedToken::FragmentBegin,
                            ExpectedToken::End,
                        ],
                    })
                    .map_err(nom::Err::Error)
            }
            _ => Err(nom::Err::Failure(ParseError2 {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::FirstQuery { prev_token } => match prev_token {
            RouteParserToken::QueryBegin => query_capture(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![ExpectedToken::QueryCapture, ExpectedToken::QueryLiteral],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::QueryCapture { .. } => alt((get_and, get_hash, get_end))(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![
                        ExpectedToken::QuerySeparator,
                        ExpectedToken::FragmentBegin,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError2 {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::NthQuery { prev_token } => match prev_token {
            RouteParserToken::QuerySeparator => query_capture(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![ExpectedToken::QueryCapture, ExpectedToken::QueryLiteral],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::QueryCapture { .. } => alt((get_and, get_hash, get_end))(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![
                        ExpectedToken::QuerySeparator,
                        ExpectedToken::FragmentBegin,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError2 {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::Fragment { prev_token } => match prev_token {
            RouteParserToken::FragmentBegin => alt((exact, capture_single, get_end))(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![
                        ExpectedToken::Literal,
                        ExpectedToken::CaptureNamed,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Exact(_) => capture_single(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![ExpectedToken::CaptureNamed],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Capture(_) => exact(i)
                .map_err(|_| ParseError2 {
                    reason: None,
                    expected: vec![ExpectedToken::CaptureNamed],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError2 {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::End => Err(nom::Err::Failure(ParseError2 {
            reason: Some(ParserErrorReason::TokensAfterEndToken),
            expected: vec![],
        })),
    }
}

fn get_slash(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('/'), |_: char| RouteParserToken::Separator)(i)
}

fn get_question(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('?'), |_: char| RouteParserToken::QueryBegin)(i)
}

fn get_and(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('&'), |_: char| RouteParserToken::QuerySeparator)(i)
}

fn get_hash(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('#'), |_: char| RouteParserToken::FragmentBegin)(i)
}

fn get_end(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('!'), |_: char| RouteParserToken::End)(i)
}

fn rust_ident(i: &str) -> IResult<&str, &str> {
    let invalid_ident_chars = r##" \|/{}[]()?+=-1234567890!@#$%^&*~`'";:"##;
    take_till1(move |c| invalid_ident_chars.contains(c))(i)
}

fn exact_impl(i: &str) -> IResult<&str, &str> {
    let special_chars = r##"/?&#={}!"##; // TODO these might allow escaping one day.
    take_till1(move |c| special_chars.contains(c))(i)
}

fn exact(i: &str) -> IResult<&str, RouteParserToken> {
    map(exact_impl, |s| RouteParserToken::Exact(s))(i)
}

fn capture(i: &str) -> IResult<&str, RouteParserToken> {
    map(capture_impl, |cv: RefCaptureVariant| {
        RouteParserToken::Capture(cv)
    })(i)
}

fn capture_single(i: &str) -> IResult<&str, RouteParserToken> {
    map(
        delimited(char('{'), single_capture_impl, char('}')),
        |cv: RefCaptureVariant| RouteParserToken::Capture(cv),
    )(i)
}

fn capture_impl(i: &str) -> IResult<&str, RefCaptureVariant> {
    let inner = alt((
        single_capture_impl,
        many_capture_impl,
        numbered_capture_impl,
    ));
    delimited(char('{'), inner, char('}'))(i)
}

fn single_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant> {
    map(rust_ident, |key| RefCaptureVariant::Named(key))(i)
}

fn many_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant> {
    map(
        separated_pair(char('*'), char(':'), rust_ident),
        |(_, key)| RefCaptureVariant::ManyNamed(key),
    )(i)
}

fn numbered_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant> {
    map(
        separated_pair(digit1, char(':'), rust_ident),
        |(number, key)| RefCaptureVariant::NumberedNamed {
            sections: number.parse().unwrap(),
            name: key,
        },
    )(i)
}

fn query_capture(i: &str) -> IResult<&str, RouteParserToken> {
    fn cap_or_exact(i: &str) -> IResult<&str, CaptureOrExact> {
        alt((
            map(
                delimited(char('{'), single_capture_impl, char('}')),
                |cap| CaptureOrExact::Capture(cap),
            ),
            map(exact_impl, |exact| CaptureOrExact::Exact(exact)),
        ))(i)
    };
    map(
        separated_pair(exact_impl, char('='), cap_or_exact),
        |(ident, capture_or_match)| RouteParserToken::QueryCapture {
            ident,
            capture_or_match,
        },
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

    mod does_parse {
        use super::*;
        #[test]
        fn query_section() {
            query_capture("lorem=ipsum").expect("should parse");
        }

        #[test]
        fn slash() {
            parse("/").expect("should parse");
        }

        #[test]
        fn slash_exact() {
            parse("/hello").expect("should parse");
        }

        #[test]
        fn multiple_exact() {
            parse("/lorem/ipsum").expect("should parse");
        }

        #[test]
        fn capture_in_path() {
            parse("/lorem/{ipsum}").expect("should parse");
        }

        #[test]
        fn capture_rest_in_path() {
            parse("/lorem/{*:ipsum}").expect("should parse");
        }

        #[test]
        fn capture_numbered_in_path() {
            parse("/lorem/{5:ipsum}").expect("should parse");
        }

        #[test]
        fn exact_query_after_path() {
            parse("/lorem?ipsum=dolor").expect("should parse");
        }

        #[test]
        fn exact_query() {
            parse("?lorem=ipsum").expect("should parse");
        }

        #[test]
        fn capture_query() {
            parse("?lorem={ipsum}").expect("should parse");
        }

        #[test]
        fn multiple_queries() {
            parse("?lorem=ipsum&dolor=sit").expect("should parse");
        }

        #[test]
        fn query_and_exact_fragment() {
            parse("?lorem=ipsum#dolor").expect("should parse");
        }

        #[test]
        fn query_with_exact_and_capture_fragment() {
            parse("?lorem=ipsum#dolor{sit}").expect("should parse");
        }

        #[test]
        fn query_with_capture_fragment() {
            parse("?lorem=ipsum#{dolor}").expect("should parse");
        }
    }

    mod does_not_parse {
        use super::*;

        #[test]
        fn empty() {
            parse("").expect_err("Should not parse");
        }

        #[test]
        fn double_slash() {
            parse("//").expect_err("Should not parse");
        }

        #[test]
        fn leading_ampersand_query() {
            parse("&query=thing").expect_err("Should not parse");
        }

        #[test]
        fn after_end() {
            parse("/lorem/ipsum!/dolor").expect_err("Should not parse");
        }

        #[test]
        fn double_end() {
            parse("/hello!!").expect_err("Should not parse");
        }

        #[test]
        fn just_end() {
            parse("!").expect_err("Should not parse");
        }
    }

    mod correct_parse {
        use super::*;

        #[test]
        fn minimal_path() {
            let parsed = parse("/lorem").unwrap();
            let expected = vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem"),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn multiple_path() {
            let parsed = parse("/lorem/ipsum/dolor/sit").unwrap();
            let expected = vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem"),
                RouteParserToken::Separator,
                RouteParserToken::Exact("ipsum"),
                RouteParserToken::Separator,
                RouteParserToken::Exact("dolor"),
                RouteParserToken::Separator,
                RouteParserToken::Exact("sit"),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn capture_path() {
            let parsed = parse("/{lorem}/{ipsum}").unwrap();
            let expected = vec![
                RouteParserToken::Separator,
                RouteParserToken::Capture(RefCaptureVariant::Named("lorem")),
                RouteParserToken::Separator,
                RouteParserToken::Capture(RefCaptureVariant::Named("ipsum")),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn query() {
            let parsed = parse("?query=this").unwrap();
            let expected = vec![
                RouteParserToken::QueryBegin,
                RouteParserToken::QueryCapture {
                    ident: "query",
                    capture_or_match: CaptureOrExact::Exact("this"),
                },
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn query_2_part() {
            let parsed = parse("?lorem=ipsum&dolor=sit").unwrap();
            let expected = vec![
                RouteParserToken::QueryBegin,
                RouteParserToken::QueryCapture {
                    ident: "lorem",
                    capture_or_match: CaptureOrExact::Exact("ipsum"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::QueryCapture {
                    ident: "dolor",
                    capture_or_match: CaptureOrExact::Exact("sit"),
                },
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn query_3_part() {
            let parsed = parse("?lorem=ipsum&dolor=sit&amet=consectetur").unwrap();
            let expected = vec![
                RouteParserToken::QueryBegin,
                RouteParserToken::QueryCapture {
                    ident: "lorem",
                    capture_or_match: CaptureOrExact::Exact("ipsum"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::QueryCapture {
                    ident: "dolor",
                    capture_or_match: CaptureOrExact::Exact("sit"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::QueryCapture {
                    ident: "amet",
                    capture_or_match: CaptureOrExact::Exact("consectetur"),
                },
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn exact_fragment() {
            let parsed = parse("#lorem").unwrap();
            let expected = vec![
                RouteParserToken::FragmentBegin,
                RouteParserToken::Exact("lorem"),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn capture_fragment() {
            let parsed = parse("#{lorem}").unwrap();
            let expected = vec![
                RouteParserToken::FragmentBegin,
                RouteParserToken::Capture(RefCaptureVariant::Named("lorem")),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn mixed_fragment() {
            let parsed = parse("#{lorem}ipsum{dolor}").unwrap();
            let expected = vec![
                RouteParserToken::FragmentBegin,
                RouteParserToken::Capture(RefCaptureVariant::Named("lorem")),
                RouteParserToken::Exact("ipsum"),
                RouteParserToken::Capture(RefCaptureVariant::Named("dolor")),
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn end_after_path() {
            let parsed = parse("/lorem!").unwrap();
            let expected = vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem"),
                RouteParserToken::End,
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn end_after_path_separator() {
            let parsed = parse("/lorem/!").unwrap();
            let expected = vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem"),
                RouteParserToken::Separator,
                RouteParserToken::End,
            ];
            assert_eq!(parsed, expected);
        }
    }
}
