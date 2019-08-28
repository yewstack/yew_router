use crate::parser::RouteParserToken;
use crate::parser::{CaptureVariant, CaptureOrMatch};
use crate::parser::parse;
use nom::error::VerboseError;

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
        match self {
            MatcherToken::Match(sequence) => Ok(&sequence),
            MatcherToken::Optional(inner) => {
                // recurse into the optional section, looking for the first
                // Match section to extract a string from.
                inner.iter()
                    .next()
                    .ok_or_else(|| ())
                    .and_then(MatcherToken::lookup_next_concrete_sequence)
            }
            _ => Err(())
        }
    }
}

/// TODO lookup_next_concrete_sequence replacement
/// Take a peekable iterator.
/// Until a top level Match is encountered, peek through optional sections.
/// If a match is found, then move the list of delimiters into a take_till seeing if the current input slice is found in the list of decimeters.
/// If a match is not found, then do the same, or accept as part of an alt() a take the rest of the input (as long as it is valid).
/// return this take_till configuration and use that to terminate / capture the given string for the capture token.




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


pub fn parse_str_and_optimize_tokens(i: &str) -> Result<Vec<MatcherToken>, nom::Err<VerboseError<&str>>> {
    let tokens = parse(i)?;
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
                // Empty the run when a optional is encountered.
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(MatcherToken::Match(s));
                    run.clear()
                }
                optimized.push(MatcherToken::Optional(optimize_tokens(tokens.clone())))
            },
            RouteParserToken::Capture (_) | RouteParserToken:: QueryCapture {..} => {
                // Empty the run when a capture is encountered.
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

