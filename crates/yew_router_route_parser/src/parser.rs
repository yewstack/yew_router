//! Parser that consumes a string and produces the first representation of the matcher.
use crate::error::{ExpectedToken, ParseError, ParserErrorReason, PrettyParseError};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until},
    character::complete::{char, digit1},
    combinator::{map, map_opt},
    sequence::{delimited, pair, separated_pair},
    IResult,
};

/// Tokens generated from parsing a route matcher string.
/// They will be optimized to another token type that is used to match URLs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteParserToken<'a> {
    /// Match /
    Separator,
    /// Match a specific string.
    Exact(&'a str),
    /// Match {_}. See `RefCaptureVariant` for more.
    Capture(RefCaptureVariant<'a>),
    /// Match ?
    QueryBegin,
    /// Match &
    QuerySeparator,
    /// Match x=y
    Query {
        /// Identifier
        ident: &'a str,
        /// Capture or match
        capture_or_exact: CaptureOrExact<'a>,
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
///
/// Its name stems from the fact that it does not have ownership over all its values.
/// It gets converted to CaptureVariant, a nearly identical enum that has owned Strings instead.
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
                RouteParserToken::Separator
                | RouteParserToken::Exact(_)
                | RouteParserToken::Capture(_) => Ok(ParserState::Path { prev_token: token }),
                RouteParserToken::QueryBegin => Ok(ParserState::FirstQuery { prev_token: token }),
                RouteParserToken::QuerySeparator // TODO this may be possible in the future.
                | RouteParserToken::Query { .. } => Err(ParserErrorReason::NotAllowedStateTransition),
                RouteParserToken::FragmentBegin => Ok(ParserState::Fragment { prev_token: token }),
                RouteParserToken::End => Ok(ParserState::End)
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
                    RouteParserToken::Query { .. } => {
                        Ok(ParserState::FirstQuery { prev_token: token })
                    }
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                RouteParserToken::Query { .. } => match token {
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
                    RouteParserToken::Query { .. } => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    _ => Err(ParserErrorReason::NotAllowedStateTransition),
                },
                RouteParserToken::Query { .. } => match token {
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
///
/// The parsing logic involves using a state machine.
/// After a token is read, this token is fed into the state machine, causing it to transition to a new state or throw an error.
/// Because the tokens that can be parsed in each state are limited, errors are not actually thrown in the state transition,
/// due to the fact that erroneous tokens can't be fed into the transition function.
///
/// This continues until the string is exhausted, or none of the parsers for the current state can parse the current input.
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
            let error = ParseError {
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
) -> IResult<&'a str, RouteParserToken<'a>, ParseError> {
    match state {
        ParserState::None => alt((get_slash, get_question, get_hash, capture, exact, get_end))(i)
            .map_err(|_| {
                get_and(i).map(|_| ParserErrorReason::AndBeforeQuestion) // TODO, technically, a sub-switch may want to start with a &query=something, so enabling this might make sense.
                    .or_else(|_| bad_capture(i).map(|(_, reason)| reason))
                    .ok()
            })
            .map_err(|reason| ParseError {
                reason,
                expected: vec![
                    ExpectedToken::Separator,
                    ExpectedToken::QueryBegin,
                    ExpectedToken::FragmentBegin,
                    ExpectedToken::CaptureManyNamed,
                ],
            })
            .map_err(nom::Err::Error),
        ParserState::Path { prev_token } => match prev_token {
            RouteParserToken::Separator => {
                alt((exact, capture, get_question, get_hash, get_end))(i)
                    .map_err(|_| {
                        // Detect likely failures if the above failed to match.
                        get_slash(i)
                            .map(|_| ParserErrorReason::DoubleSlash)
                            .or_else(|_| get_and(i).map(|_| ParserErrorReason::AndBeforeQuestion))
                            .or_else(|_| bad_capture(i).map(|(_, reason)| reason))
                            .ok()
                    })
                    .map_err(|reason| ParseError {
                        reason,
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
                    .map_err(|_| {
                        get_and(i)
                            .map(|_| ParserErrorReason::AndBeforeQuestion)
                            .or_else(|_| bad_capture(i).map(|(_, reason)| reason))
                            .ok()
                    })
                    .map_err(|reason| ParseError {
                        reason,
                        expected: vec![
                            ExpectedToken::Separator,
                            ExpectedToken::QueryBegin,
                            ExpectedToken::FragmentBegin,
                            ExpectedToken::End,
                            ExpectedToken::CaptureNamed,
                            ExpectedToken::CaptureManyNamed,
                            ExpectedToken::CaptureNumberedNamed,
                        ],
                    })
                    .map_err(nom::Err::Error)
            }
            RouteParserToken::Capture(_) => alt((get_slash, exact, get_question, get_hash))(i)
                .map_err(|_| {
                    capture(i)
                        .map(|_| ParserErrorReason::AdjacentCaptures)
                        .or_else(|_| get_and(i).map(|_| ParserErrorReason::AndBeforeQuestion))
                        .or_else(|_| get_end(i).map(|_| ParserErrorReason::EndAfterCapture))
                        .ok()
                })
                .map_err(|reason| ParseError {
                    reason,
                    expected: vec![
                        ExpectedToken::Literal,
                        ExpectedToken::Separator,
                        ExpectedToken::QueryBegin,
                        ExpectedToken::FragmentBegin,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::FirstQuery { prev_token } => match prev_token {
            RouteParserToken::QueryBegin => query(i)
                .map_err(|_| {
                    get_question(i)
                        .map(|_| ParserErrorReason::MultipleQuestions)
                        .ok()
                })
                .map_err(|reason| ParseError {
                    reason,
                    expected: vec![ExpectedToken::QueryCapture, ExpectedToken::QueryLiteral],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Query { .. } => alt((get_and, get_hash, get_end))(i)
                .map_err(|_| {
                    get_question(i)
                        .map(|_| ParserErrorReason::MultipleQuestions)
                        .ok()
                })
                .map_err(|reason| ParseError {
                    reason,
                    expected: vec![
                        ExpectedToken::QuerySeparator,
                        ExpectedToken::FragmentBegin,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::NthQuery { prev_token } => match prev_token {
            RouteParserToken::QuerySeparator => query(i)
                .map_err(|_| {
                    get_question(i)
                        .map(|_| ParserErrorReason::MultipleQuestions)
                        .ok()
                })
                .map_err(|reason| ParseError {
                    reason,
                    expected: vec![ExpectedToken::QueryCapture, ExpectedToken::QueryLiteral],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Query { .. } => {
                alt((get_and, get_hash, get_end))(i) // TODO only allow ends after "literals"
                    .map_err(|_| {
                        get_question(i)
                            .map(|_| ParserErrorReason::MultipleQuestions)
                            .ok()
                    })
                    .map_err(|reason| ParseError {
                        reason,
                        expected: vec![
                            ExpectedToken::QuerySeparator,
                            ExpectedToken::FragmentBegin,
                            ExpectedToken::End,
                        ],
                    })
                    .map_err(nom::Err::Error)
            }
            _ => Err(nom::Err::Failure(ParseError {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::Fragment { prev_token } => match prev_token {
            RouteParserToken::FragmentBegin => alt((exact, capture_single, get_end))(i)
                .map_err(|_| ParseError {
                    reason: None,
                    expected: vec![
                        ExpectedToken::Literal,
                        ExpectedToken::CaptureNamed,
                        ExpectedToken::End,
                    ],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Exact(_) => alt((capture_single, get_end))(i)
                .map_err(|_| ParseError {
                    reason: None,
                    expected: vec![ExpectedToken::CaptureNamed, ExpectedToken::End],
                })
                .map_err(nom::Err::Error),
            RouteParserToken::Capture(_) => exact(i)
                .map_err(|_| bad_capture(i).map(|(_, reason)| reason).ok())
                .map_err(|reason| ParseError {
                    reason,
                    expected: vec![ExpectedToken::CaptureNamed],
                })
                .map_err(nom::Err::Error),
            _ => Err(nom::Err::Failure(ParseError {
                reason: Some(ParserErrorReason::InvalidState),
                expected: vec![],
            })),
        },
        ParserState::End => Err(nom::Err::Failure(ParseError {
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

/// Returns a FragmentBegin variant if the next character is '\#'.
fn get_hash(i: &str) -> IResult<&str, RouteParserToken> {
    map(char('#'), |_: char| RouteParserToken::FragmentBegin)(i)
}

/// Returns an End variant if the next character is a '!`.
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
    map(named_capture_impl, |cv: RefCaptureVariant| {
        RouteParserToken::Capture(cv)
    })(i)
}

fn capture_single(i: &str) -> IResult<&str, RouteParserToken> {
    map(
        delimited(char('{'), single_capture_impl, char('}')),
        RouteParserToken::Capture,
    )(i)
}

/// Captures {ident}, {*:ident}, {<number>:ident}
fn named_capture_impl(i: &str) -> IResult<&str, RefCaptureVariant> {
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

/// Gets a capture or exact, mapping it to the CaptureOrExact enum - to provide a limited subset.
fn cap_or_exact(i: &str) -> IResult<&str, CaptureOrExact> {
    alt((
        map(
            delimited(char('{'), single_capture_impl, char('}')),
            CaptureOrExact::Capture,
        ),
        map(exact_impl, |exact| CaptureOrExact::Exact(exact)),
    ))(i)
}

/// Matches a query
fn query(i: &str) -> IResult<&str, RouteParserToken> {
    map(
        separated_pair(exact_impl, char('='), cap_or_exact),
        |(ident, capture_or_exact)| RouteParserToken::Query {
            ident,
            capture_or_exact,
        },
    )(i)
}

/// Succeeds if an invalid character is used as an ident.
fn bad_capture(i: &str) -> IResult<&str, ParserErrorReason> {
    let invalid_ident_chars = r##" \|/{[]()?+=-1234567890!@#$%^&*~`'";:"##;

    let number_capture = map(
        separated_pair(digit1, char(':'), take_until("}")),
        |(_, ident)| ident,
    );
    let many_capture = map(pair(tag("*:"), take_until("}")), |(_, ident)| ident);
    let simple_capture = take_until("}");
    map_opt(
        delimited(
            char('{'),
            alt((number_capture, many_capture, simple_capture)),
            char('}'),
        ),
        move |s: &str| {
            s.chars()
                .map(|ch| {
                    if invalid_ident_chars.contains(ch) {
                        Some(ParserErrorReason::BadRustIdent(ch))
                    } else {
                        None
                    }
                })
                .flatten()
                .next()
        },
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

    mod sub_parsers {
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
        fn bad_capture_with_ampersand() {
            bad_capture("{ident&}").expect("should parse");
        }

        #[test]
        fn bad_capture_approves_valid_idents() {
            bad_capture("{ident}").expect_err("should not parse");
            bad_capture("{*:ident}").expect_err("should not parse");
            bad_capture("{2:ident}").expect_err("should not parse");
        }
    }

    mod does_parse {
        use super::*;

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

        // TODO, should empty be ok?
        #[test]
        fn empty() {
            parse("").expect_err("Should not parse");
        }

        #[test]
        fn double_slash() {
            let x = parse("//").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::DoubleSlash))
        }

        #[test]
        fn slash_ampersand() {
            let x = parse("/&lorem=ipsum").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::AndBeforeQuestion))
        }

        #[test]
        fn non_ident_capture() {
            let x = parse("/{lor#m}").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::BadRustIdent('#')))
        }

        #[test]
        fn leading_ampersand_query() {
            let x = parse("&query=thing").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::AndBeforeQuestion));
        }

        #[test]
        fn after_end() {
            let x = parse("/lorem/ipsum!/dolor").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::TokensAfterEndToken));
        }

        #[test]
        fn double_end() {
            let x = parse("/hello!!").expect_err("Should not parse");
            assert_eq!(x.error.reason, Some(ParserErrorReason::TokensAfterEndToken));
        }
    }

    mod correct_parse {
        use super::*;

        #[test]
        fn starting_literal() {
            let parsed = parse("lorem").unwrap();
            let expected = vec![RouteParserToken::Exact("lorem")];
            assert_eq!(parsed, expected);
        }

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
                RouteParserToken::Query {
                    ident: "query",
                    capture_or_exact: CaptureOrExact::Exact("this"),
                },
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn query_2_part() {
            let parsed = parse("?lorem=ipsum&dolor=sit").unwrap();
            let expected = vec![
                RouteParserToken::QueryBegin,
                RouteParserToken::Query {
                    ident: "lorem",
                    capture_or_exact: CaptureOrExact::Exact("ipsum"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::Query {
                    ident: "dolor",
                    capture_or_exact: CaptureOrExact::Exact("sit"),
                },
            ];
            assert_eq!(parsed, expected);
        }

        #[test]
        fn query_3_part() {
            let parsed = parse("?lorem=ipsum&dolor=sit&amet=consectetur").unwrap();
            let expected = vec![
                RouteParserToken::QueryBegin,
                RouteParserToken::Query {
                    ident: "lorem",
                    capture_or_exact: CaptureOrExact::Exact("ipsum"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::Query {
                    ident: "dolor",
                    capture_or_exact: CaptureOrExact::Exact("sit"),
                },
                RouteParserToken::QuerySeparator,
                RouteParserToken::Query {
                    ident: "amet",
                    capture_or_exact: CaptureOrExact::Exact("consectetur"),
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

        #[test]
        fn just_end() {
            let parsed = parse("!").unwrap();
            assert_eq!(parsed, vec![RouteParserToken::End]);
        }
    }
}
