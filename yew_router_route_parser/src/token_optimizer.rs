use crate::parser::RouteParserToken;
use crate::parser::{CaptureVariant, CaptureOrMatch};
use crate::parser::parse;
use nom::error::VerboseError;
use std::iter::Peekable;
use nom::IResult;
use std::slice::Iter;
use nom::bytes::complete::take_till1;
use nom::combinator::{peek, rest, cond, map, map_opt};
use crate::parser::util::alternative;
use nom::branch::alt;

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

/// Produces a parser combinator that searches for the next possible set of strings of
/// characters used to terminate a forward search.
///
/// Take a peekable iterator.
/// Until a top level Match is encountered, peek through optional sections.
/// If a match is found, then move the list of delimiters into a take_till seeing if the current input slice is found in the list of decimeters.
/// If a match is not found, then do the same, or accept as part of an alt() a take the rest of the input (as long as it is valid).
/// return this take_till configuration and use that to terminate / capture the given string for the capture token.
pub fn next_delimiters<'a>(mut iter:  Peekable<Iter<MatcherToken>>) -> impl Fn(&'a str) -> IResult<&'a str, &'a str>  {

    enum MatchOrOptSequence<'a> {
        Match(&'a str),
        Optional(&'a str)
    }
    fn search_for_inner_sequence(matcher_token: &MatcherToken) -> Option<&str> {
        match matcher_token {
            MatcherToken::Match(sequence) => Some(&sequence),
            MatcherToken::Optional(inner) => {
                inner
                    .iter()
                    .filter_map(|inner_token| {
                        if let Some(inner_sequence) = search_for_inner_sequence(inner_token) {
                            Some(inner_sequence)
                        } else {
                            None
                        }
                    })
                    .next()
            }
            MatcherToken::Capture(_) => None // TODO still may want to handle this
        }
    }

    let mut sequences = vec![];
    while let Some(next) = iter.next() {
        match next {
            MatcherToken::Match(sequence) =>  {
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
            _ => panic!("underlying parser should not allow token order not of match or optional")
        }
    }

    let contains_optional = sequences.iter().any(|x| std::mem::discriminant(x) == std::mem::discriminant(&&MatchOrOptSequence::Optional("")));
    log::trace!("next delimiter: contains optional: {}", contains_optional);
    let delimiters: Vec<String> = sequences
        .into_iter()
        .map(|s| {
            match s {
                MatchOrOptSequence::Match(s) => s,
                MatchOrOptSequence::Optional(s) => s
            }
        })
        .map(String::from)
        .collect();

    log::trace!("delimiters in read_until_next_known_delimiter: {:?}", delimiters);

    // if the sequence contains an optional section, it can attempt to match until the end.
    map_opt (
        alt((cond(true, alternative(delimiters)), cond(contains_optional, rest))),
        |x| x
    )
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

