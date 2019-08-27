
pub use yew_router_route_parser::{MatcherToken, CaptureVariant, FromMatches, FromMatchesError};

mod match_paths;

use nom::IResult;
use std::collections::{HashMap, HashSet};
use log::{trace};
use yew_router_route_parser::{optimize_tokens, parser};
use yew::{Html, Component, Renderable};

pub trait RenderFn<CTX: yew::Component>: Fn(&Matches) -> Option<Html<CTX>> {}


impl <CTX, T> RenderFn<CTX> for T
    where
    T: Fn(&Matches) -> Option<Html<CTX>>,
    CTX: yew::Component
{}

pub type Matches<'a> = HashMap<&'a str, String>;

/// Attempts to match routes, transform the route to Component props and render that Component.
///
/// The CTX refers to the context of the parent rendering this (The Router).
#[derive(Debug, PartialEq, Clone)]
pub struct PathMatcher {
    pub tokens: Vec<MatcherToken>,
}

impl PathMatcher {

    pub fn try_from<CMP>(i: &str) -> Result<Self, ()> // TODO: Error handling
        where
            CMP: Component + Renderable<CMP>,
            CMP::Properties: FromMatches
    {
        let tokens = parser::parse(i).map_err(|_| ())?;
        let pm = PathMatcher {
            tokens: optimize_tokens(tokens),
        };
        Ok(pm)
    }

    // TODO, should find some way to support '/' characters in fragment. In the transform function, it could keep track of the seen hash begin yet, and transform captures based on that.
    pub fn match_path<'a>(&self, i: &'a str) -> IResult<&'a str, Matches> {
        match_paths::match_paths(&self.tokens, i)
    }

    /// Gets a set of all names that will be captured.
    /// This is useful in determining if a given struct will be able to be populated by a given path matcher before being given a concrete path to match.
    pub fn capture_names(&self) -> HashSet<&str> {
        fn capture_names_impl(tokens: &[MatcherToken]) -> HashSet<&str> {
            tokens.iter().fold(HashSet::new(), |mut acc: HashSet<&str>, token| {
                match token {
                    MatcherToken::Optional(t) => {
                        let captures = capture_names_impl(&t);
                        acc.extend(captures)
                    } ,
                    MatcherToken::Match(_) => {}
                    MatcherToken::Capture(variant)  => {
                        match variant {
                            CaptureVariant::ManyNamed(name) | CaptureVariant::Named(name) | CaptureVariant::NumberedNamed {name, ..} => {acc.insert(&name);},
                            CaptureVariant::ManyUnnamed | CaptureVariant::Unnamed | CaptureVariant::NumberedUnnamed {..} => {}
                        }
                    },
                }
                acc
            })
        }
        capture_names_impl(&self.tokens)
    }
}






    #[cfg(test)]
mod tests {
    use super::*;
    use yew_router_route_parser::parser::RouteParserToken;

    impl From<Vec<RouteParserToken>> for PathMatcher {
        fn from(tokens: Vec<RouteParserToken>) -> Self {
            PathMatcher {
                tokens: optimize_tokens(tokens),
            }
        }
    }

    #[test]
    fn basic_separator() {
        let tokens = vec![RouteParserToken::Separator];
        let path_matcher = PathMatcher::from(tokens);
        path_matcher.match_path("/").expect("should parse");
    }

    #[test]
    fn multiple_tokens() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Match("hello".to_string()), RouteParserToken::Separator];

        let path_matcher = PathMatcher::from(tokens);
        path_matcher.match_path("/hello/").expect("should parse");
    }


    #[test]
    fn simple_capture() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::Named("hello".to_string())), RouteParserToken::Separator];
        let path_matcher = PathMatcher::from(tokens);
        let (_, matches) = path_matcher.match_path("/general_kenobi/").expect("should parse");
        assert_eq!(matches["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn simple_capture_with_no_trailing_separator() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::Named("hello".to_string()))];
        let path_matcher = PathMatcher::from(tokens);
        let (_, matches) = path_matcher.match_path("/general_kenobi").expect("should parse");
        assert_eq!(matches["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn match_with_trailing_match_any() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Match("a".to_string()), RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::Unnamed)];
        let path_matcher = PathMatcher::from(tokens);
        let (_, _matches) = path_matcher.match_path("/a/").expect("should parse");
    }

    #[test]
    fn match_n() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::NumberedUnnamed {sections: 3}), RouteParserToken::Separator, RouteParserToken::Match("a".to_string())];
        let path_matcher = PathMatcher::from(tokens);
        let (_, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_n_no_overrun() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::NumberedUnnamed {sections: 3})];
        let path_matcher = PathMatcher::from(tokens);
        let (s, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3").expect("should parse");
        assert_eq!(s.len(), 0)
    }


    #[test]
    fn match_n_named() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::NumberedNamed {sections: 3, name: "captured".to_string() }), RouteParserToken::Separator, RouteParserToken::Match("a".to_string())];
        let path_matcher = PathMatcher::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }


    #[test]
    fn match_many() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::ManyUnnamed), RouteParserToken::Separator, RouteParserToken::Match("a".to_string())];
        let path_matcher = PathMatcher::from(tokens);
        let (_, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_many_named() {
        let tokens = vec![RouteParserToken::Separator, RouteParserToken::Capture(CaptureVariant::ManyNamed("captured".to_string())), RouteParserToken::Separator, RouteParserToken::Match("a".to_string())];
        let path_matcher = PathMatcher::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }

}