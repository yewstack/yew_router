
pub use yew_router_route_parser::{OptimizedToken, CaptureVariants, FromMatches, FromMatchesError};

use nom::IResult;
use std::collections::{HashMap, HashSet};
use nom::bytes::complete::{tag, take_until, is_not};
use nom::sequence::{preceded, terminated};
use nom::combinator::peek;
use log::{trace, debug};
use yew_router_route_parser::{optimize_tokens, new_parser};
use yew::{Html, Component, Renderable};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Error};
use yew::virtual_dom::{VComp, VNode, vcomp::ScopeHolder};

pub trait RenderFn<CTX: yew::Component>: Fn(&HashMap<&str, String>) -> Option<Html<CTX>> + objekt::Clone {}

impl <CTX, T> RenderFn<CTX> for T
    where
    T: Fn(&HashMap<&str, String>) -> Option<Html<CTX>> + objekt::Clone,
    CTX: yew::Component
{}

/// Attempts to match routes, transform the route to Component props and render that Component.
///
/// The CTX refers to the context of the parent rendering this (The Router).
pub struct PathMatcher<CTX: Component + Renderable<CTX>> {
    pub tokens: Vec<OptimizedToken>,
    pub render_fn: Option<Box<dyn RenderFn<CTX>>> // Having Router specified here would make dependency issues appear.
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




fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does, I may want a scope holder directly tied to the current context?
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}




impl <CTX: Component + Renderable<CTX>> PathMatcher<CTX> {

    pub fn try_from<CMP>(i: &str, cmp: PhantomData<CMP>) -> Result<Self, ()>
        where
            CMP: Component + Renderable<CMP>,
            CMP::Properties: FromMatches
    {
        let (_i, tokens) = new_parser::parse(i).map_err(|_| ())?;
        let pm = PathMatcher {
            tokens: optimize_tokens(tokens),
            render_fn: Some(Self::create_render_fn(cmp))
        };
        Ok(pm)
    }

    pub fn create_render_fn<CMP>(_: PhantomData<CMP>) -> Box<dyn RenderFn<CTX>>
        where
            CMP: Component + Renderable<CMP>,
            CMP::Properties: FromMatches
    {
        Box::new(|matches: &HashMap<&str, String>| {
            CMP::Properties::from_matches(matches)
                .map(|properties| create_component::<CMP, CTX>(properties))
                .map_err(|err| {
                    trace!("Component could not be created from matches: {:?}", err);
                })
                .ok()
        })
    }


    // TODO, should find some way to support '/' characters in fragment. In the transform function, it could keep track of the seen hash begin yet, and transform captures based on that.
    pub fn match_path<'a>(&self, mut i: &'a str) -> IResult<&'a str, HashMap<&str, String>> {
        debug!("Attempting to match path: {:?} using: {:?}", i, self);

        let mut iter = self.tokens
            .iter()
            .peekable();

        let mut matches: HashMap<&str, String> = HashMap::new();

        while let Some(token) = iter.next() {
            i = match token {
                OptimizedToken::Match(literal) => {
                    trace!("Matching literal: {}", literal);
                    tag(literal.as_str())(i)?.0
                },

                OptimizedToken::Capture (variant) => {
                    match variant {
                        CaptureVariants::Unnamed => {
                            log::trace!("Matching Unnamed");
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
                            trace!("Matching ManyUnnamed");
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
                                take_until(delimiter)(i)?.0
                            } else {
                                if i.len() == 0 {
                                    i // Match even if nothing is left
                                } else {
                                    valid_many_capture_characters(i)?.0
                                }
                            }
                        }
                        CaptureVariants::NumberedUnnamed { sections } => {
                            log::trace!("Matching NumberedUnnamed ({})", sections);
                            let mut sections = *sections;
                            if let Some(peaked_next_token) = iter.peek() {
                                while sections > 0 {
                                    if sections > 1 {
                                        i = terminated(valid_capture_characters, tag("/"))(i)?.0;
                                    } else {
                                        // Don't consume the next character on the last section
                                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                                        i = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?.0;
                                    }
                                    sections -= 1;
                                }
                            } else {
                                while sections > 0 {
                                    if sections > 1 {
                                        i = terminated(valid_capture_characters, tag("/"))(i)?.0;
                                    } else {
                                        // Don't consume the next character on the last section
                                        i = valid_capture_characters(i)?.0;
                                    }
                                    sections -= 1;
                                }
                            }
                            i
                        }
                        CaptureVariants::Named(capture_key) => {
                            log::trace!("Matching Named ({})", capture_key);
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                                let (ii, captured) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
                                matches.insert(&capture_key, captured.to_string());
                                ii
                            } else {
                                let (ii, captured) = valid_capture_characters(i)?;
                                matches.insert(&capture_key, captured.to_string());
                                ii
                            }
                        }
                        CaptureVariants::ManyNamed(capture_key) => {
                            log::trace!("Matching NumberedUnnamed ({})", capture_key);
                            if let Some(peaked_next_token) = iter.peek() {
                                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
                                let (ii, c) = take_until(delimiter)(i)?;
                                matches.insert(&capture_key, c.to_string());
                                ii
                            } else {
                                if i.len() == 0 {
                                    matches.insert(&capture_key, "".to_string()); // Is this a thing I want?
                                    i // Match even if nothing is left
                                } else {
                                    let (ii, c) = valid_many_capture_characters(i)?;
                                    matches.insert(&capture_key, c.to_string());
                                    ii
                                }
                            }
                        }
                        CaptureVariants::NumberedNamed { sections, name } => {
                            log::trace!("Matching NumberedNamed ({}, {})", sections, name);
                            let mut sections = *sections;
                            let mut captured = "".to_string();
                            if let Some(peaked_next_token) = iter.peek() {
                                while sections > 0 {
                                    if sections > 1 {
                                        let (ii, c) = terminated(valid_capture_characters, tag("/"))(i)?;
                                        i = ii;
                                        captured += c;
                                        captured += "/";
                                    } else {
                                        // Don't consume the next character on the last section
                                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                                        let (ii, c) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
                                        i = ii;
                                        captured += c;
                                    }
                                    sections -= 1;
                                    println!("{}", i);
                                }
                            } else {
                                while sections > 0 {
                                    if sections > 1 {
                                        let (ii, c) = terminated(valid_capture_characters, tag("/"))(i)?;
                                        i = ii;
                                        captured += c;
                                    } else {
                                        // Don't consume the next character on the last section
                                        let (ii, c) = valid_capture_characters(i)?;
                                        i = ii;
                                        captured += c;
                                    }
                                    sections -= 1;
                                    println!("{}", i);
                                }
                            }
                            matches.insert(&name, captured);
                            i
                        }
                    }
                },
                OptimizedToken::QueryCapture { ident,  value: capture_key} => {
                    if let Some(peaked_next_token) = iter.peek() {
                        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                        let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), take_until(delimiter))(i)?; // TODO this should also probably prevent invalid characters
                        matches.insert(&capture_key, captured.to_string());
                        ii
                    } else {
                        let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), valid_capture_characters_in_query)(i)?;
                        matches.insert(&capture_key, captured.to_string());
                        ii
                    }
                },
            };
        }
        debug!("Path Matched");

        Ok((i, matches))
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





#[cfg(test)]
mod tests {
    use super::*;
    use yew_router_route_parser::new_parser::Token;
    use yew::ComponentLink;

    struct DummyC {}
    impl Component for DummyC {
        type Message = ();
        type Properties = ();
        fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
            unimplemented!()
        }
        fn update(&mut self, msg: Self::Message) -> bool {
            unimplemented!()
        }
    }
    impl Renderable<DummyC> for DummyC {
        fn view(&self) -> VNode<Self> {
            unimplemented!()
        }
    }

    impl <CTX: Component + Renderable<CTX>> From<Vec<Token>> for PathMatcher<CTX> {
        fn from(tokens: Vec<Token>) -> Self {
            PathMatcher {
                tokens: optimize_tokens(tokens),
                render_fn: None
            }
        }
    }

    #[test]
    fn basic_separator() {
        let tokens = vec![Token::Separator];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        path_matcher.match_path("/").expect("should parse");
    }

    #[test]
    fn multiple_tokens() {
        let tokens = vec![Token::Separator, Token::Match("hello".to_string()), Token::Separator];

        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        path_matcher.match_path("/hello/").expect("should parse");
    }


    #[test]
    fn simple_capture() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::Named("hello".to_string())), Token::Separator];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/general_kenobi/").expect("should parse");
        assert_eq!(matches["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn simple_capture_with_no_trailing_separator() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::Named("hello".to_string()))];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, dict) = path_matcher.match_path("/general_kenobi").expect("should parse");
        assert_eq!(dict["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn match_with_trailing_match_any() {
        let tokens = vec![Token::Separator, Token::Match("a".to_string()), Token::Separator, Token::Capture(CaptureVariants::Unnamed)];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, dict) = path_matcher.match_path("/a/").expect("should parse");
    }

    #[test]
    fn match_n() {
        let tokens = vec![Token::Separator,  Token::Capture(CaptureVariants::NumberedUnnamed {sections: 3}), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, dict) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_n_no_overrun() {
        let tokens = vec![Token::Separator,  Token::Capture(CaptureVariants::NumberedUnnamed {sections: 3})];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (s, dict) = path_matcher.match_path("/garbage1/garbage2/garbage3").expect("should parse");
        assert_eq!(s.len(), 0)
    }


    #[test]
    fn match_n_named() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::NumberedNamed {sections: 3, name: "captured".to_string() }), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }


    #[test]
    fn match_many() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::ManyUnnamed), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_many_named() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariants::ManyNamed("captured".to_string())), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }

}