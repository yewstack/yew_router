use crate::parser::Token;

use nom::IResult;
use std::collections::{HashMap, HashSet};
use nom::bytes::complete::{tag, take_until, is_not};
use nom::sequence::{preceded, terminated};
use std::convert::TryFrom;
use nom::combinator::peek;
use log::debug;

fn token_to_string(token: &Token) -> &str {
    match token {
        Token::Separator => "/",
        Token::Match(literal) => &literal,
        Token::QueryBegin => "?",
        Token::QuerySeparator => "&",
        Token::FragmentBegin => "#",
        Token::MatchAny | Token::Capture {..} | Token::QueryCapture {..} => {
            log::error!("Bout to crash!");
            unreachable!()
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct PathMatcher {
    pub tokens: Vec<OptimizedToken>
}

impl TryFrom<&str> for PathMatcher {
    type Error = ();

    fn try_from(i: &str) -> Result<Self, Self::Error> {
        let (_i, tokens) = crate::parser::parse(i).map_err(|_| ())?;
        Ok(PathMatcher::from(tokens))
    }
}


// TODO this apparently doesn't work?.
// Its not super important.
use nom::Err;
/// Should assign i.
#[allow(unused)]
fn assign_i<I,O,E>(f: impl Fn(I) -> Result<(I, O), Err<E>>) -> impl Fn(I) -> Result<O, Err<E>> {
    move |mut i| {
        let (ii, x) = f(i)?;
        i = ii;
        Ok(x)
    }
}

impl From<Vec<Token>> for PathMatcher {
    fn from(tokens: Vec<Token>) -> Self {
        let mut optimized = vec![];
        let mut run = vec![];

        tokens.into_iter().for_each( |token| {
            match &token {
                Token::Separator | Token::Match(_) | Token::QueryBegin | Token::QuerySeparator | Token::FragmentBegin => {
                    run.push(token)
                }
                Token::MatchAny | Token::Capture {..} | Token:: QueryCapture {..} => {
                    if !run.is_empty() {
                        let s: String = run.iter().map(token_to_string).collect();
                        optimized.push(OptimizedToken::Match(s));
                        run.clear()
                    }
                    let token = match token {
                        Token::MatchAny => OptimizedToken::MatchAny,
                        Token::Capture {ident} => OptimizedToken::Capture {ident},
                        Token::QueryCapture {ident, value} => OptimizedToken::QueryCapture {ident, value},
                        _ => {
                            log::error!("crashing time");
                            unreachable!()
                        }
                    };
                    optimized.push(token);
                }
            }
        });
        if !run.is_empty() {
            let s: String = run.iter().map(token_to_string).collect();
            optimized.push(OptimizedToken::Match(s));
        }

        PathMatcher {
            tokens: optimized
        }
    }
}

impl PathMatcher {
    // TODO, should find some way to support '/' characters in fragment. In the transform function, it could keep track of the seen hash begin yet, and transform captures based on that.
    pub fn match_path<'a>(&self, mut i: &'a str) -> IResult<&'a str, HashMap<String, String>> {
        debug!("Attempting to match path: {:?} using: {:?}", i, self);
        let mut iter = self.tokens
            .iter()
            .peekable();

        let mut dictionary: HashMap<String, String> = HashMap::new();

        while let Some(token) = iter.next() {
//            dbg!(i);
            i = match token {
                OptimizedToken::Match(literal) => {
                    log::debug!("Matching literal: {}", literal);
                    tag(literal.as_str())(i)?.0
                },
                OptimizedToken::MatchAny => {
                    log::debug!("Matching any");
                    if let Some(peaked_next_token) = iter.peek() {
                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
                        terminated(valid_capture_characters, peek(tag(delimiter)))(i)?.0
                    } else {
                        valid_capture_characters(i)?.0
                    }
                },
                OptimizedToken::Capture { ident: capture_key } => {
                    if let Some(peaked_next_token) = iter.peek() {
                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                        dbg!(delimiter);
                        let (ii, captured) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
                        dictionary.insert(capture_key.clone(), captured.to_string());
                        ii
                    } else {
                        let (ii, captured) = valid_capture_characters(i)?;
                        dictionary.insert(capture_key.clone(), captured.to_string());
                        ii
                    }
                },
                OptimizedToken::QueryCapture { ident,  value: capture_key} => {
                    if let Some(peaked_next_token) = iter.peek() {
                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                        let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), take_until(delimiter))(i)?; // TODO this should also probably prevent invalid characters
                        dictionary.insert(capture_key.clone(), captured.to_string());
                        ii
                    } else {
                        let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), valid_capture_characters_in_query)(i)?;
                        dictionary.insert(capture_key.clone(), captured.to_string());
                        ii
                    }
                },
            };
        }
        debug!("Path Matched");

        Ok((i, dictionary))
    }

    /// Gets a set of all names that will be captured.
    /// This is useful in determining if a given struct will be able to be populated by a given path matcher before being given a concrete path to match.
    pub fn capture_names(&self) -> HashSet<&str> {
        self.tokens.iter().fold(HashSet::new(), |mut acc, token| {
            match token {
                OptimizedToken::Match(_) | OptimizedToken::MatchAny => {}
                OptimizedToken::Capture { ident  } => {acc.insert(ident);},
                OptimizedToken::QueryCapture {ident, ..} => {acc.insert(ident);},
            }
            acc
        })

    }
}


/// Characters that don't interfere with parsing logic for capturing characters
fn valid_capture_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " */#&?|{}=";
    is_not(INVALID_CHARACTERS)(i)
}

fn valid_capture_characters_in_query(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " *#&?|{}=";
    is_not(INVALID_CHARACTERS)(i)
}

#[derive(Debug, PartialEq)]
pub enum OptimizedToken {
    /// Extraneous section-related tokens can be condensed into a match.
    Match(String),
    MatchAny,
    Capture{ ident: String},
    QueryCapture {
        ident: String,
        value: String
    }
}

impl OptimizedToken {
    /// Helper method to get concrete literals out of Match variants.
    fn lookup_next_concrete_sequence(&self) -> Result<&str, ()> {
        if let OptimizedToken::Match(sequence) = self {
            Ok(&sequence)
        } else {
            Err(())
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_separator() {
        let tokens = vec![Token::Separator];
        let path_matcher = PathMatcher::from(tokens);
        path_matcher.match_path("/").expect("should parse");
    }

    #[test]
    fn multiple_tokens() {
        let tokens = vec![Token::Separator, Token::Match("hello".to_string()), Token::Separator];

        let path_matcher = PathMatcher::from(tokens);
        path_matcher.match_path("/hello/").expect("should parse");
    }


    #[test]
    fn simple_capture() {
        let tokens = vec![Token::Separator, Token::Capture { ident: "hello".to_string() }, Token::Separator];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/general_kenobi/").expect("should parse");
        assert_eq!(dict.get(&"hello".to_string()), Some(&"general_kenobi".to_string()))
    }


    #[test]
    fn simple_capture_with_no_trailing_separator() {
        let tokens = vec![Token::Separator, Token::Capture { ident: "hello".to_string() }];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/general_kenobi").expect("should parse");
        assert_eq!(dict.get(&"hello".to_string()), Some(&"general_kenobi".to_string()))
    }


    #[test]
    fn match_with_trailing_match_any() {
        let tokens = vec![Token::Separator, Token::Match("a".to_string()), Token::Separator, Token::MatchAny];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/a/").expect("should parse");
    }
}
