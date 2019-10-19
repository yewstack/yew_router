//! Parser that consumes a string and produces the first representation of the matcher.
use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::complete::{char, digit1},
    combinator::map,
    sequence::{delimited, separated_pair},
    IResult,
};

mod optimizer;
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
    fn transition(self, token: RouteParserToken<'a>) -> Result<Self, ParserError> {
        match self {
            ParserState::None => match token {
                RouteParserToken::Separator => Ok(ParserState::Path { prev_token: token }),
                RouteParserToken::Exact(_) => Err(ParserError::NotAllowedStateTransition),
                RouteParserToken::Capture(_) => Ok(ParserState::Path { prev_token: token }), /* TODO revise decision to allow this state transform for _all_ capture variants. */
                RouteParserToken::QueryBegin => Ok(ParserState::FirstQuery { prev_token: token }),
                RouteParserToken::QuerySeparator => Err(ParserError::NotAllowedStateTransition),
                RouteParserToken::QueryCapture { .. } => Err(ParserError::NotAllowedStateTransition),
                RouteParserToken::FragmentBegin => Ok(ParserState::Fragment { prev_token: token }),
                RouteParserToken::End => Err(ParserError::NotAllowedStateTransition),
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
                        _ => Err(ParserError::NotAllowedStateTransition),
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
                        _ => Err(ParserError::NotAllowedStateTransition),
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
                        _ => Err(ParserError::NotAllowedStateTransition),
                    },
                    _ => Err(ParserError::InvalidState), /* Other previous token types are
                                                          * invalid within a Path state. */
                }
            }
            ParserState::FirstQuery { prev_token } => match prev_token {
                RouteParserToken::QueryBegin => match token {
                    RouteParserToken::QueryCapture { .. } => {
                        Ok(ParserState::FirstQuery { prev_token: token })
                    }
                    _ => Err(ParserError::NotAllowedStateTransition),
                },
                RouteParserToken::QueryCapture { .. } => match token {
                    RouteParserToken::QuerySeparator => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    RouteParserToken::FragmentBegin => {
                        Ok(ParserState::Fragment { prev_token: token })
                    }
                    RouteParserToken::End => Ok(ParserState::End),
                    _ => Err(ParserError::NotAllowedStateTransition),
                },
                _ => Err(ParserError::InvalidState),
            },
            ParserState::NthQuery { prev_token } => match prev_token {
                RouteParserToken::QuerySeparator => match token {
                    RouteParserToken::QueryCapture { .. } => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    _ => Err(ParserError::NotAllowedStateTransition),
                },
                RouteParserToken::QueryCapture { .. } => match token {
                    RouteParserToken::QuerySeparator => {
                        Ok(ParserState::NthQuery { prev_token: token })
                    }
                    RouteParserToken::FragmentBegin => {
                        Ok(ParserState::Fragment { prev_token: token })
                    }
                    RouteParserToken::End => Ok(ParserState::End),
                    _ => Err(ParserError::NotAllowedStateTransition),
                },
                _ => Err(ParserError::InvalidState),
            },
            ParserState::Fragment { prev_token } => match prev_token {
                RouteParserToken::FragmentBegin
                | RouteParserToken::Exact(_)
                | RouteParserToken::Capture(_) => Ok(ParserState::Fragment { prev_token: token }),
                RouteParserToken::End => Ok(ParserState::End),
                _ => Err(ParserError::InvalidState),
            },
            ParserState::End => Err(ParserError::TokensAfterEndToken),
        }
    }
}

/// Something went wrong with parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    /// The parser should not be able to be in this state with this token
    InvalidState,
    /// The state cannot transition from a prior state into another one based on the input token.
    NotAllowedStateTransition, // TODO replace this with more exact explanations
    /// Some token encountered after the end token.
    TokensAfterEndToken,
    /// Two slashes are able to ocurr next to eachother.
    DoubleSlash,
    /// A & appears before a ?
    AndBeforeQuestion,
    /// Expected a /
    ExpectedSlash,
    /// The parser expected one of the following sequences.
    ExpectedOneOf(Vec<RouteParserToken<'static>>),
}

/// Parse a matching string into a vector of RouteParserTokens.
pub fn parse(mut i: &str) -> Result<Vec<RouteParserToken>, (&str, ParserError)> {
    let mut tokens: Vec<RouteParserToken> = vec![];
    let mut state = ParserState::None;

    loop {
        let (ii, token) = parse_impl(i, &state).map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => (i, e),
            _ => panic!("parser should not be incomplete"),
        })?;
        i = ii;
        state = state.transition(token.clone()).map_err(|e| (i, e))?;
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
) -> IResult<&'a str, RouteParserToken<'a>, ParserError> {
    match state {
        ParserState::None => alt((get_slash, get_question, get_hash, capture))(i).map_err(|_| {
            nom::Err::Error(ParserError::ExpectedOneOf(vec![
                RouteParserToken::Separator,
                RouteParserToken::QueryBegin,
                RouteParserToken::FragmentBegin,
            ]))
        }),
        ParserState::Path { prev_token } => match prev_token {
            RouteParserToken::Separator => {
                alt((exact, capture, get_question, get_hash, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::Exact(""),
                        RouteParserToken::Capture(RefCaptureVariant::Named("")),
                        RouteParserToken::Capture(RefCaptureVariant::ManyNamed("")),
                        RouteParserToken::Capture(RefCaptureVariant::NumberedNamed {
                            sections: 0,
                            name: "",
                        }),
                        RouteParserToken::QueryBegin,
                        RouteParserToken::FragmentBegin,
                        RouteParserToken::End,
                    ]))
                })
            }
            RouteParserToken::Exact(_) => {
                alt((get_slash, capture, get_question, get_hash, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::Separator,
                        RouteParserToken::Capture(RefCaptureVariant::Named("")),
                        RouteParserToken::Capture(RefCaptureVariant::ManyNamed("")),
                        RouteParserToken::Capture(RefCaptureVariant::NumberedNamed {
                            sections: 0,
                            name: "",
                        }),
                        RouteParserToken::QueryBegin,
                        RouteParserToken::FragmentBegin,
                        RouteParserToken::End,
                    ]))
                })
            }
            RouteParserToken::Capture(_) => {
                alt((get_slash, exact, get_question, get_hash, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::Separator,
                        RouteParserToken::Capture(RefCaptureVariant::Named("")),
                        RouteParserToken::Capture(RefCaptureVariant::ManyNamed("")),
                        RouteParserToken::Capture(RefCaptureVariant::NumberedNamed {
                            sections: 0,
                            name: "",
                        }),
                        RouteParserToken::QueryBegin,
                        RouteParserToken::FragmentBegin,
                        RouteParserToken::End,
                    ]))
                })
            }
            _ => Err(nom::Err::Failure(ParserError::InvalidState)),
        },
        ParserState::FirstQuery { prev_token } => match prev_token {
            RouteParserToken::QueryBegin => query_capture(i).map_err(|_| {
                nom::Err::Error(ParserError::ExpectedOneOf(vec![
                    RouteParserToken::QueryCapture {
                        ident: "",
                        capture_or_match: CaptureOrExact::Capture(RefCaptureVariant::Named("")),
                    },
                ]))
            }),
            RouteParserToken::QueryCapture { .. } => {
                alt((get_and, get_hash, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::QuerySeparator,
                        RouteParserToken::FragmentBegin,
                        RouteParserToken::End,
                    ]))
                })
            }
            _ => Err(nom::Err::Failure(ParserError::InvalidState)),
        },
        ParserState::NthQuery { prev_token } => match prev_token {
            RouteParserToken::QuerySeparator => query_capture(i).map_err(|_| {
                nom::Err::Error(ParserError::ExpectedOneOf(vec![
                    RouteParserToken::QueryCapture {
                        ident: "",
                        capture_or_match: CaptureOrExact::Capture(RefCaptureVariant::Named("")),
                    },
                ]))
            }),
            RouteParserToken::QueryCapture { .. } => {
                alt((get_and, get_hash, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::QuerySeparator,
                        RouteParserToken::FragmentBegin,
                        RouteParserToken::End,
                    ]))
                })
            }
            _ => Err(nom::Err::Failure(ParserError::InvalidState)),
        },
        ParserState::Fragment { prev_token } => match prev_token {
            RouteParserToken::FragmentBegin => {
                alt((exact, capture_single, get_end))(i).map_err(|_| {
                    nom::Err::Error(ParserError::ExpectedOneOf(vec![
                        RouteParserToken::Exact(""),
                        RouteParserToken::Capture(RefCaptureVariant::Named("")),
                        RouteParserToken::End,
                    ]))
                })
            }
            RouteParserToken::Exact(_) => capture_single(i).map_err(|_| {
                nom::Err::Error(ParserError::ExpectedOneOf(vec![RouteParserToken::Capture(
                    RefCaptureVariant::Named(""),
                )]))
            }),
            RouteParserToken::Capture(_) => exact(i).map_err(|_| {
                nom::Err::Error(ParserError::ExpectedOneOf(vec![RouteParserToken::Capture(
                    RefCaptureVariant::Named(""),
                )]))
            }),
            _ => Err(nom::Err::Failure(ParserError::InvalidState)),
        },
        ParserState::End => Err(nom::Err::Failure(ParserError::TokensAfterEndToken)),
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
    map(char('!'), |_: char| RouteParserToken::FragmentBegin)(i)
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
