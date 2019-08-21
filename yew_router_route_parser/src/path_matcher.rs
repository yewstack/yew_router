use crate::new_parser::Token;
use crate::new_parser::CaptureVariants;

use nom::IResult;
use std::collections::{HashMap, HashSet};
use nom::bytes::complete::{tag, take_until, is_not};
use nom::sequence::{preceded, terminated};
use std::convert::TryFrom;
use nom::combinator::peek;
use log::debug;
use yew::{Html, Component, Renderable};
use crate::FromMatches;

fn token_to_string(token: &Token) -> &str {
    match token {
        Token::Separator => "/",
        Token::Match(literal) => &literal,
        Token::QueryBegin => "?",
        Token::QuerySeparator => "&",
        Token::FragmentBegin => "#",
        Token::Capture {..} | Token::QueryCapture {..} => {
//            log::error!("Bout to crash!");
            unreachable!()
        }
    }
}

/// The CTX refers to the context of the parent rendering this (The Router).
pub struct PathMatcher<CTX: Component + Renderable<CTX>> {
    pub tokens: Vec<OptimizedToken>,
    pub render_fn: Box<dyn Fn(&HashMap<String, String>) -> Option<Html<CTX>>> // Having Router specified here would make dependency issues appear.
}

impl <CTX: Component + Renderable<CTX>> PartialEq for PathMatcher<CTX> {
    fn eq(&self, other: &Self) -> bool {
        self.tokens.eq(&other.tokens) && std::ptr::eq(&self.render_fn, &other.render_fn)
    }
}

impl <CTX: Component + Renderable<CTX>> Debug for PathMatcher<CTX> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("PathMatcher")
            .field("tokens", &self.tokens)
            .field("render_fn", &"Fn".to_string())
            .finish()
    }
}
//
//impl <CTX: FromMatches> TryFrom<&str> for PathMatcher<CTX> {
//    type Error = ();
//
//    fn try_from(i: &str) -> Result<Self, Self::Error> {
//        let (_i, tokens) = crate::new_parser::parse(i).map_err(|_| ())?;
//        let pm = PathMatcher {
//            tokens: optimize_tokens(tokens),
//            render_fn: Box::new(())
//        };
//        Ok((tokens))
//    }
//}






// TODO this apparently doesn't work?.
// Its not super important.
use nom::Err;
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Error};
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::{VComp, VNode};

/// Should assign i.
#[allow(unused)]
fn assign_i<I,O,E>(f: impl Fn(I) -> Result<(I, O), Err<E>>) -> impl Fn(I) -> Result<O, Err<E>> {
    move |mut i| {
        let (ii, x) = f(i)?;
        i = ii;
        Ok(x)
    }
}

// TODO, forget the whole From thing. just create a free function that creates a vec<OptimizedToken>
// Then the html fn can be created separately.


//impl <CTX: FromMatches, TARGET> From<Vec<Token>> for PathMatcher<CTX> {
//    fn from(tokens: Vec<Token>) -> Self {
//
//
//        PathMatcher {
//            tokens: optimized,
//            render_fn: Box::new(<<TARGET as Component>::Properties> as FromMatches::from_matches)
//        }
//    }
//}

fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does, I may want a scope holder directly tied to the current context?
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}

pub fn parse_str_and_optimize_tokens(i: &str) -> Result<Vec<OptimizedToken>, ()> {
    let (_, tokens) = crate::new_parser::parse(i).map_err(|_| ())?;
    Ok(optimize_tokens(tokens))
}

fn optimize_tokens(tokens: Vec<Token>) -> Vec<OptimizedToken> {
    let mut optimized = vec![];
    let mut run = vec![];

    tokens.into_iter().for_each( |token| {
        match &token {
            Token::Separator | Token::Match(_) | Token::QueryBegin | Token::QuerySeparator | Token::FragmentBegin => {
                run.push(token)
            }
            Token::Capture (_) | Token:: QueryCapture {..} => {
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(OptimizedToken::Match(s));
                    run.clear()
                }
                let token = match token {
                    Token::Capture (variant) => OptimizedToken::Capture (variant),
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
    optimized
}


impl <CTX: Component + Renderable<CTX>> PathMatcher<CTX> {



    pub fn try_from<CMP>(i: &str, cmp: PhantomData<CMP>) -> Result<Self, ()>
    where
        CMP: Component + Renderable<CMP>,
        CMP::Properties: FromMatches
    {
        let (_i, tokens) = crate::new_parser::parse(i).map_err(|_| ())?;
        let pm = PathMatcher {
            tokens: optimize_tokens(tokens),
            render_fn: Self::create_render_fn(cmp)
        };
        Ok(pm)
    }

    pub fn create_render_fn<CMP>(_: PhantomData<CMP>) -> Box<Fn(&HashMap<String, String>) -> Option<Html<CTX>>>
    where
        CMP: Component + Renderable<CMP>,
        CMP::Properties: FromMatches
    {
        Box::new(|matches: &HashMap<String, String>| {
            CMP::Properties::from_matches(matches)
                .map(|properties| create_component::<CMP, CTX>(properties))
                .map_err(|err| {
                    debug!("Component could not be created from matches: {:?}", err);
                })
                .ok()
        })
    }


    // TODO, should find some way to support '/' characters in fragment. In the transform function, it could keep track of the seen hash begin yet, and transform captures based on that.
    pub fn match_path<'a>(&self, mut i: &'a str) -> IResult<&'a str, HashMap<String, String>> {
        debug!("Attempting to match path: {:?} using: {:?}", i, self);
        let mut iter = self.tokens
            .iter()
            .peekable();

        let mut dictionary: HashMap<String, String> = HashMap::new();

        while let Some(token) = iter.next() {
            i = match token {
                OptimizedToken::Match(literal) => {
                    log::debug!("Matching literal: {}", literal);
                    tag(literal.as_str())(i)?.0
                },

                OptimizedToken::Capture (variant) => {
                    match variant {
                        CaptureVariants::Unnamed => {
                            log::debug!("Matching Unnamed");
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
                                terminated(valid_capture_characters, peek(tag(delimiter)))(i)?.0
                            } else {
                                if i.len() == 0 {
                                    i // Match even if nothing is left
                                } else if i == "/" {
                                    "" // Trailing '/' is allowed
                                } else {
                                    valid_capture_characters(i)?.0
                                }
                            }
                        },
                        CaptureVariants::ManyUnnamed => {
                            log::debug!("Matching ManyUnnamed");
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
                                terminated(valid_many_capture_characters, peek(tag(delimiter)))(i)?.0
                            } else {
                                if i.len() == 0 {
                                    i // Match even if nothing is left
                                } else {
                                    valid_many_capture_characters(i)?.0
                                }
                            }
                        }
                        CaptureVariants::NumberedUnnamed { sections: _ } => {
                            unimplemented!()
                        }
                        CaptureVariants::Named(capture_key) => {
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                                let (ii, captured) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
                                dictionary.insert(capture_key.clone(), captured.to_string());
                                ii
                            } else {
                                let (ii, captured) = valid_capture_characters(i)?;
                                dictionary.insert(capture_key.clone(), captured.to_string());
                                ii
                            }
                        }
                        CaptureVariants::ManyNamed(_) => {
                            unimplemented!()
                        }
                        CaptureVariants::NumberedNamed { sections: _, name: _ } => {
                            unimplemented!()
                        }
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
                OptimizedToken::Match(_) => {}
                OptimizedToken::Capture(variant)  => {
                    match variant {
                        CaptureVariants::ManyNamed(name) | CaptureVariants::Named(name) | CaptureVariants::NumberedNamed {name, ..} => {acc.insert(name);},
                        CaptureVariants::ManyUnnamed | CaptureVariants::Unnamed | CaptureVariants::NumberedUnnamed {..} => {}
                    }
                },
                OptimizedToken::QueryCapture {ident, ..} => {
                    acc.insert(ident);
                },
            }
            acc
        })

    }
}


/// Characters that don't interfere with parsing logic for capturing characters
fn valid_capture_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " */#&?{}=";
    is_not(INVALID_CHARACTERS)(i)
}

fn valid_many_capture_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " #&?{}=";
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
    Capture(CaptureVariants),
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
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::Named("hello".to_string())), Token::Separator];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/general_kenobi/").expect("should parse");
        assert_eq!(dict.get(&"hello".to_string()), Some(&"general_kenobi".to_string()))
    }


    #[test]
    fn simple_capture_with_no_trailing_separator() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::Named("hello".to_string()))];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/general_kenobi").expect("should parse");
        assert_eq!(dict.get(&"hello".to_string()), Some(&"general_kenobi".to_string()))
    }


    #[test]
    fn match_with_trailing_match_any() {
        let tokens = vec![Token::Separator, Token::Match("a".to_string()), Token::Separator, Token::Capture(CaptureVariants::Unnamed)];
        let path_matcher = PathMatcher::from(tokens);
        let (_, dict) = path_matcher.match_path("/a/").expect("should parse");
    }
}
