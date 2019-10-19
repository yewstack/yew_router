//! Logic for matching and capturing route strings.

pub use yew_router_route_parser::{CaptureVariant, Captures, MatcherToken};

mod route_matcher;
pub use self::route_matcher::{MatcherSettings, RouteMatcher};
