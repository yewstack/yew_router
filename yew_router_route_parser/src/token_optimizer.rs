use crate::parser::RouteParserToken;
use crate::parser::{CaptureVariant, CaptureOrMatch};
use crate::parser::parse;

/// Tokens used to determine how to match and capture sections from a URL.
#[derive(Debug, PartialEq, Clone)]
pub enum MatcherToken {
    /// Extraneous section-related tokens can be condensed into a match.
    Match(String),
    Capture(CaptureVariant),
    Optional(Vec<MatcherToken>)
}


impl From<CaptureOrMatch> for MatcherToken {
    fn from(value: CaptureOrMatch) -> Self {
        match value {
            CaptureOrMatch::Match(m) => MatcherToken::Match(m),
            CaptureOrMatch::Capture(v) => MatcherToken::Capture(v)
        }
    }
}

impl MatcherToken {
    /// Helper method to get concrete literals out of Match variants.
    pub fn lookup_next_concrete_sequence(&self) -> Result<&str, ()> {
        if let MatcherToken::Match(sequence) = self {
            Ok(&sequence)
        } else {
            Err(())
        }
    }
}


/// Tokens that can be coalesced to a OptimizedToken::Match are converted to strings here.
fn token_to_string(token: &RouteParserToken) -> &str {
    match token {
        RouteParserToken::Separator => "/",
        RouteParserToken::Match(literal) => &literal,
        RouteParserToken::QueryBegin => "?",
        RouteParserToken::QuerySeparator => "&",
        RouteParserToken::FragmentBegin => "#",
        RouteParserToken::Capture {..} | RouteParserToken::QueryCapture {..} | RouteParserToken::Optional(_)=> {
            unreachable!()
        }
    }
}


pub fn parse_str_and_optimize_tokens(i: &str) -> Result<Vec<MatcherToken>, ()> {
    let tokens = parse(i).map_err(|_| ())?;
    Ok(optimize_tokens(tokens))
}

pub fn optimize_tokens(tokens: Vec<RouteParserToken>) -> Vec<MatcherToken> {
    // The list of optimized tokens.
    let mut optimized = vec![];
    // Stores consecutive Tokens that can be reduced down to a OptimizedToken::Match.
    let mut run = vec![];

    tokens.into_iter().for_each( |token| {
        match &token {
            RouteParserToken::Separator | RouteParserToken::Match(_) | RouteParserToken::QueryBegin | RouteParserToken::QuerySeparator | RouteParserToken::FragmentBegin => {
                run.push(token)
            }
            RouteParserToken::Optional(tokens) => {
                optimized.push(MatcherToken::Optional(optimize_tokens(tokens.clone())))
            },
            RouteParserToken::Capture (_) | RouteParserToken:: QueryCapture {..} => {
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(MatcherToken::Match(s));
                    run.clear()
                }
                match token {
                    RouteParserToken::Capture (variant) => {
                        optimized.push(MatcherToken::Capture (variant))
                    },
                    RouteParserToken::QueryCapture {ident, capture_or_match} => {
                        optimized.extend(vec![MatcherToken::Match(format!("{}=", ident)), capture_or_match.into()])
                    }
                    _ => {
                        log::error!("crashing time");
                        unreachable!()
                    }
                };
            }
        }
    });
    // empty the "run"
    if !run.is_empty() {
        let s: String = run.iter().map(token_to_string).collect();
        optimized.push(MatcherToken::Match(s));
    }
    optimized
}

