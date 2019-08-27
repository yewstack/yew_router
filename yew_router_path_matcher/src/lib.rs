
pub use yew_router_route_parser::{OptimizedToken, CaptureVariant, FromMatches, FromMatchesError};

mod match_paths;

use nom::IResult;
use std::collections::{HashMap, HashSet};
use log::{trace};
use yew_router_route_parser::{optimize_tokens, parser};
use yew::{Html, Component, Renderable};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Error};
use yew::virtual_dom::{VComp, VNode, vcomp::ScopeHolder};

pub trait RenderFn<CTX: yew::Component>: Fn(&Matches) -> Option<Html<CTX>> + objekt::Clone {}

pub type Matches<'a> = HashMap<&'a str, String>;

impl <CTX, T> RenderFn<CTX> for T
    where
    T: Fn(&Matches) -> Option<Html<CTX>> + objekt::Clone,
    CTX: yew::Component
{}

/// Attempts to match routes, transform the route to Component props and render that Component.
///
/// The CTX refers to the context of the parent rendering this (The Router).
pub struct PathMatcher {
    pub tokens: Vec<OptimizedToken>,
//    pub render_fn: Option<Box<dyn RenderFn<CTX>>> // Having Router specified here would make dependency issues appear.
}

impl PartialEq for PathMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.tokens.eq(&other.tokens) //&& std::ptr::eq(&self.render_fn, &other.render_fn)
    }
}

impl Debug for PathMatcher {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("PathMatcher")
            .field("tokens", &self.tokens)
//            .field("render_fn", &"Fn".to_string())
            .finish()
    }
}




fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does, I may want a scope holder directly tied to the current context?
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}



impl PathMatcher {

    pub fn try_from<CMP>(i: &str, cmp: PhantomData<CMP>) -> Result<Self, ()>
        where
            CMP: Component + Renderable<CMP>,
            CMP::Properties: FromMatches
    {
        let (_i, tokens) = parser::parse(i).map_err(|_| ())?;
        let pm = PathMatcher {
            tokens: optimize_tokens(tokens),
//            render_fn: Some(Self::create_render_fn(cmp))
        };
        Ok(pm)
    }

//    pub fn create_render_fn<CMP>(_: PhantomData<CMP>) -> Box<dyn RenderFn<CTX>>
//        where
//            CMP: Component + Renderable<CMP>,
//            CMP::Properties: FromMatches
//    {
//        Box::new(|matches: &HashMap<&str, String>| {
//            CMP::Properties::from_matches(matches)
//                .map(|properties| create_component::<CMP, CTX>(properties))
//                .map_err(|err| {
//                    trace!("Component could not be created from matches: {:?}", err);
//                })
//                .ok()
//        })
//    }


    // TODO, should find some way to support '/' characters in fragment. In the transform function, it could keep track of the seen hash begin yet, and transform captures based on that.
    pub fn match_path<'a>(&self, i: &'a str) -> IResult<&'a str, Matches> {
        match_paths::match_paths(&self.tokens, i)
    }

    /// Gets a set of all names that will be captured.
    /// This is useful in determining if a given struct will be able to be populated by a given path matcher before being given a concrete path to match.
    pub fn capture_names(&self) -> HashSet<&str> {
        self.tokens.iter().fold(HashSet::new(), |mut acc, token| {
            match token {
                OptimizedToken::Optional(_) => unimplemented!("TODO need ability to recurse"),
                OptimizedToken::Match(_) => {}
                OptimizedToken::Capture(variant)  => {
                    match variant {
                        CaptureVariant::ManyNamed(name) | CaptureVariant::Named(name) | CaptureVariant::NumberedNamed {name, ..} => {acc.insert(name);},
                        CaptureVariant::ManyUnnamed | CaptureVariant::Unnamed | CaptureVariant::NumberedUnnamed {..} => {}
                    }
                },
            }
            acc
        })
    }
}







#[cfg(test)]
mod tests {
    use super::*;
    use yew_router_route_parser::parser::Token;
    use yew::ComponentLink;

    struct DummyC {}
    impl Component for DummyC {
        type Message = ();
        type Properties = ();
        fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
            unimplemented!()
        }
        fn update(&mut self, _msg: Self::Message) -> bool {
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
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::Named("hello".to_string())), Token::Separator];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/general_kenobi/").expect("should parse");
        assert_eq!(matches["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn simple_capture_with_no_trailing_separator() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::Named("hello".to_string()))];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/general_kenobi").expect("should parse");
        assert_eq!(matches["hello"], "general_kenobi".to_string())
    }


    #[test]
    fn match_with_trailing_match_any() {
        let tokens = vec![Token::Separator, Token::Match("a".to_string()), Token::Separator, Token::Capture(CaptureVariant::Unnamed)];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, _matches) = path_matcher.match_path("/a/").expect("should parse");
    }

    #[test]
    fn match_n() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::NumberedUnnamed {sections: 3}), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_n_no_overrun() {
        let tokens = vec![Token::Separator,  Token::Capture(CaptureVariant::NumberedUnnamed {sections: 3})];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (s, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3").expect("should parse");
        assert_eq!(s.len(), 0)
    }


    #[test]
    fn match_n_named() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::NumberedNamed {sections: 3, name: "captured".to_string() }), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }


    #[test]
    fn match_many() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::ManyUnnamed), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, _matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
    }

    #[test]
    fn match_many_named() {
        let tokens = vec![Token::Separator, Token::Capture(CaptureVariant::ManyNamed("captured".to_string())), Token::Separator, Token::Match("a".to_string())];
        let path_matcher = PathMatcher::<DummyC>::from(tokens);
        let (_, matches) = path_matcher.match_path("/garbage1/garbage2/garbage3/a").expect("should parse");
        assert_eq!(matches["captured"], "garbage1/garbage2/garbage3".to_string())
    }

}