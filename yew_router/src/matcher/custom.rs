//! Custom matcher implementation support via dynamic trait objects.
use std::fmt;
use std::rc::Rc;
use crate::matcher::{Matcher, MatcherProvider};


/// Wrapper for a user-defined matcher implementation.
///
/// # Example
/// ```
///# use yew_router::matcher::{MatcherProvider, Matcher};
///# use yew_router_route_parser::Captures;
///# use yew_router::matcher::CustomMatcher;
///# use std::rc::Rc;
/// struct ExactMatcher(String);
/// impl MatcherProvider for ExactMatcher {
///     fn match_route_string<'a, 'b: 'a>(&'b self,route_string: &'a str) -> Option<Captures> {
///         if &self.0 == route_string {
///             Some(Captures::new())
///         } else {
///             None
///         }
///     }
/// }
/// impl ExactMatcher {
///     fn into_matcher(self) -> Matcher {
///         Matcher::from(Rc::new(self) as Rc<dyn MatcherProvider>)
///     }
/// }
/// ```
#[derive(Clone)]
pub struct CustomMatcher(Rc<dyn MatcherProvider>);

impl fmt::Debug for CustomMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CustomMatcher")
    }
}

impl PartialEq for CustomMatcher {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl From<Rc<dyn MatcherProvider>> for CustomMatcher {
    fn from(value: Rc<dyn MatcherProvider>) -> Self {
        CustomMatcher(value)
    }
}

impl From<Rc<dyn MatcherProvider>> for Matcher {
    fn from(value: Rc<dyn MatcherProvider>) -> Self {
        Matcher::CustomMatcher(CustomMatcher(value))
    }
}

impl std::ops::Deref for CustomMatcher {
    type Target = Rc<dyn MatcherProvider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}