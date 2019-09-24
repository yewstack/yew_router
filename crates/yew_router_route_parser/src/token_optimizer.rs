use crate::parser::parse;
use crate::parser::RouteParserToken;
use crate::parser::{Capture, CaptureOrExact};
use nom::IResult;
use std::iter::Peekable;
use std::slice::Iter;
//use nom::bytes::complete::take_till1;
use crate::parser::util::alternative;
use crate::parser::YewRouterParseError;
use nom::branch::alt;
use nom::combinator::{cond, map_opt, rest};

/// Tokens used to determine how to match and capture sections from a URL.
#[derive(Debug, PartialEq, Clone)]
pub enum MatcherToken {
    /// Section-related tokens can be condensed into a match.
    Exact(String),
    /// Capture section.
    Capture(Capture),
    /// Section that doesn't have to match.
    Optional(Vec<MatcherToken>),
}

impl From<CaptureOrExact> for MatcherToken {
    fn from(value: CaptureOrExact) -> Self {
        match value {
            CaptureOrExact::Exact(m) => MatcherToken::Exact(m),
            CaptureOrExact::Capture(v) => MatcherToken::Capture(v),
        }
    }
}

/// Produces a parser combinator that searches for the next possible set of strings of
/// characters used to terminate a forward search.
///
/// Take a peekable iterator.
/// Until a top level Match is encountered, peek through optional sections.
/// If a match is found, then move the list of delimiters into a take_till seeing if the current input slice is found in the list of decimeters.
/// If a match is not found, then do the same, or accept as part of an alt() a take the rest of the input (as long as it is valid).
/// return this take_till configuration and use that to terminate / capture the given string for the capture token.
pub fn next_delimiters<'a>(
    iter: Peekable<Iter<MatcherToken>>,
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    enum MatchOrOptSequence<'a> {
        Match(&'a str),
        Optional(&'a str),
    }
    fn search_for_inner_sequence(matcher_token: &MatcherToken) -> Option<&str> {
        match matcher_token {
            MatcherToken::Exact(sequence) => Some(&sequence),
            MatcherToken::Optional(inner) => inner
                .iter()
                .filter_map(|inner_token| {
                    if let Some(inner_sequence) = search_for_inner_sequence(inner_token) {
                        Some(inner_sequence)
                    } else {
                        None
                    }
                })
                .next(),
            MatcherToken::Capture(_) => None, // TODO still may want to handle this
        }
    }

    let mut sequences = vec![];
    for next in iter {
        match next {
            MatcherToken::Exact(sequence) => {
                sequences.push(MatchOrOptSequence::Match(&sequence));
                break;
            }
            MatcherToken::Optional(inner) => {
                let sequence: &str = inner
                    .iter()
                    .filter_map(search_for_inner_sequence)
                    .next()
                    .expect("Should be in sequence");
                sequences.push(MatchOrOptSequence::Optional(sequence))
            }
            _ => panic!("underlying parser should not allow token order not of match or optional"),
        }
    }

    let contains_optional = sequences.iter().any(|x| {
        std::mem::discriminant(x) == std::mem::discriminant(&&MatchOrOptSequence::Optional(""))
    });
    log::trace!("next delimiter: contains optional: {}", contains_optional);
    let delimiters: Vec<String> = sequences
        .into_iter()
        .map(|s| match s {
            MatchOrOptSequence::Match(s) => s,
            MatchOrOptSequence::Optional(s) => s,
        })
        .map(String::from)
        .collect();

    log::trace!(
        "delimiters in read_until_next_known_delimiter: {:?}",
        delimiters
    );

    // if the sequence contains an optional section, it can attempt to match until the end.
    map_opt(
        alt((
            cond(true, alternative(delimiters)),
            cond(contains_optional, rest),
        )),
        |x| x,
    )
}

/// Tokens that can be coalesced to a OptimizedToken::Match are converted to strings here.
fn token_to_string(token: &RouteParserToken) -> &str {
    match token {
        RouteParserToken::Separator => "/",
        RouteParserToken::Exact(literal) => &literal,
        RouteParserToken::QueryBegin => "?",
        RouteParserToken::QuerySeparator => "&",
        RouteParserToken::FragmentBegin => "#",
        RouteParserToken::Capture { .. }
        | RouteParserToken::QueryCapture { .. }
        | RouteParserToken::Optional(_) => unreachable!(),
    }
}

/// Parse the provided "matcher string" and then optimize the tokens.
pub fn parse_str_and_optimize_tokens(
    i: &str,
    append_optional_slash: bool,
) -> Result<Vec<MatcherToken>, YewRouterParseError> {
    let tokens = parse(i)?;
    Ok(optimize_tokens(tokens, append_optional_slash))
}

/// Optimize `RouteParserToken`s to `MatcherToken`s.
///
/// This involves condensing sequential tokens that represent statically knowable characters into large `Match` tokens.
/// For example, the tokens \[Separator, Match("thing"), Separator\] becomes just \[Match("/thing/")\].
///
/// It also if configured to do so, will add optional slashes at the end of path sections where appropriate.
pub fn optimize_tokens(
    tokens: Vec<RouteParserToken>,
    append_optional_slash: bool,
) -> Vec<MatcherToken> {
    // The list of optimized tokens.
    let mut optimized: Vec<MatcherToken> = vec![];
    // Stores consecutive Tokens that can be reduced down to a OptimizedToken::Match.
    let mut run: Vec<RouteParserToken> = vec![];

    let mut fragment_or_query_encountered = false;

    let mut token_iterator = tokens.into_iter().peekable();

    while let Some(token) = token_iterator.next() {
        match &token {
            RouteParserToken::QueryBegin | RouteParserToken::FragmentBegin => {
                fragment_or_query_encountered = true;
                run.push(token)
            }
            RouteParserToken::Separator | RouteParserToken::QuerySeparator => run.push(token),
            RouteParserToken::Exact(_) => {
                run.push(token);

                // Only append the optional slash if:
                if append_optional_slash // the settings allow it
                    && !fragment_or_query_encountered // There hasn't been a fragment or query
                    && token_is_not_present_or_is_either_a_slash_or_question(token_iterator.peek())
                // The next token doesn't exist or is a '/' or '?'
                {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(MatcherToken::Exact(s));
                    run.clear();
                    optimized.push(MatcherToken::Optional(vec![MatcherToken::Exact(
                        "/".to_string(),
                    )]))
                }
            }

            RouteParserToken::Optional(tokens) => {
                // Empty the run when a optional is encountered.
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(MatcherToken::Exact(s));
                    run.clear()
                }

                optimized.push(MatcherToken::Optional(optimize_tokens(
                    tokens.clone(),
                    false,
                )));

                if append_optional_slash {
                    // If the optional is the last token (at this level of nesting), then stick a optional (/) at the end
                    if token_iterator.peek().is_none() {
                        // Safety: its fine to unconditionally add another optional slash here,
                        // because optional sections SHOULD_NOT be able to be parsed with a trailing '/'
                        optimized.push(MatcherToken::Optional(vec![MatcherToken::Exact(
                            "/".to_string(),
                        )]))
                    }
                }
            }
            RouteParserToken::Capture(variant) => {
                // Empty the run when a capture is encountered.
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(MatcherToken::Exact(s));
                    run.clear()
                }
                optimized.push(MatcherToken::Capture(variant.clone()))
            }
            RouteParserToken::QueryCapture {
                ident,
                capture_or_match,
            } => {
                run.push(RouteParserToken::Exact(format!("{}=", ident))); // Push the ident to the run either way.
                match capture_or_match {
                    CaptureOrExact::Exact(m) => run.push(RouteParserToken::Exact(m.clone())),
                    CaptureOrExact::Capture(capture) => {
                        let s: String = run.iter().map(token_to_string).collect();
                        optimized.push(MatcherToken::Exact(s));
                        run.clear();

                        optimized.push(MatcherToken::Capture(capture.clone()))
                    }
                }
            }
        }
    }
    // empty the "run".
    if !run.is_empty() {
        let s: String = run.iter().map(token_to_string).collect();
        optimized.push(MatcherToken::Exact(s));
    }
    optimized
}

fn token_is_not_present_or_is_either_a_slash_or_question(token: Option<&RouteParserToken>) -> bool {
    match token {
        None | Some(RouteParserToken::QueryBegin) | Some(RouteParserToken::FragmentBegin) => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::CaptureVariant;

    #[test]
    fn conversion_cap_or_exact_to_matcher_token_exact() {
        let mt = MatcherToken::from(CaptureOrExact::Exact("lorem".to_string()));
        assert_eq!(mt, MatcherToken::Exact("lorem".to_string()))
    }

    #[test]
    fn conversion_cap_or_exact_to_matcher_token_capture() {
        let mt = MatcherToken::from(CaptureOrExact::Capture(Capture::from(
            CaptureVariant::Unnamed,
        )));
        assert_eq!(
            mt,
            MatcherToken::Capture(Capture::from(CaptureVariant::Unnamed))
        )
    }

    #[test]
    fn optimization_inserts_optional_slash_at_end() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("thing".to_string()),
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("/thing".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimization_inserts_optional_slash_before_query() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("thing".to_string()),
            RouteParserToken::QueryBegin,
            RouteParserToken::QueryCapture {
                ident: "HelloThere".to_string(),
                capture_or_match: CaptureOrExact::Exact("GeneralKenobi".to_string()),
            },
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("/thing".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
            MatcherToken::Exact("?HelloThere=GeneralKenobi".to_string()),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimization_inserts_optional_slash_before_fragment() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("thing".to_string()),
            RouteParserToken::FragmentBegin,
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("/thing".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
            MatcherToken::Exact("#".to_string()),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimization_does_not_insert_optional_slash_after_slash() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("thing".to_string()),
            RouteParserToken::Separator,
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![MatcherToken::Exact("/thing/".to_string())];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimize_optional() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("lorem".to_string()),
            RouteParserToken::Optional(vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem".to_string()),
            ]),
        ];
        let optimized = optimize_tokens(tokens, false);
        let expected = vec![
            MatcherToken::Exact("/lorem".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/lorem".to_string())]),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimize_optional_with_optional_slash() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("lorem".to_string()),
            RouteParserToken::Optional(vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("lorem".to_string()),
            ]),
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("/lorem".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/lorem".to_string())]),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimize_capture_all() {
        let tokens = vec![RouteParserToken::Capture(Capture::from(
            CaptureVariant::ManyNamed("lorem".to_string()),
        ))];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![MatcherToken::Capture(Capture::from(
            CaptureVariant::ManyNamed("lorem".to_string()),
        ))];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimize_capture_everything_after_initial_slash() {
        let tokens = vec![
            RouteParserToken::Separator,
            RouteParserToken::Capture(Capture::from(CaptureVariant::ManyNamed(
                "lorem".to_string(),
            ))),
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("/".to_string()),
            MatcherToken::Capture(Capture::from(CaptureVariant::ManyNamed(
                "lorem".to_string(),
            ))),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn optimize_query_capture() {
        let tokens = vec![
            RouteParserToken::QueryBegin,
            RouteParserToken::QueryCapture {
                ident: "lorem".to_string(),
                capture_or_match: CaptureOrExact::Capture(Capture::from(CaptureVariant::Unnamed)),
            },
        ];
        let optimized = optimize_tokens(tokens, true);
        let expected = vec![
            MatcherToken::Exact("?lorem=".to_string()),
            MatcherToken::Capture(Capture::from(CaptureVariant::Unnamed)),
        ];
        assert_eq!(expected, optimized);
    }

    #[test]
    fn next_delimiter_simple() {
        let tokens = vec![MatcherToken::Exact("/".to_string())];
        next_delimiters(tokens.iter().peekable())("/").expect("should match");
    }

    #[test]
    fn next_delimiter_optional() {
        let tokens = vec![
            MatcherToken::Exact("/thing".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        next_delimiters(tokens.iter().peekable())("/thing").expect("should match");
        next_delimiters(tokens.iter().peekable())("/thing/").expect("should match");
    }

    #[test]
    fn next_delimiter_nested_optional() {
        let tokens = vec![
            MatcherToken::Exact("/thing".to_string()),
            MatcherToken::Optional(vec![MatcherToken::Optional(vec![MatcherToken::Exact(
                "/".to_string(),
            )])]),
        ];
        next_delimiters(tokens.iter().peekable())("/thing").expect("should match");
        next_delimiters(tokens.iter().peekable())("/thing/").expect("should match");
    }

}
