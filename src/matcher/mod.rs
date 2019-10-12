//! Logic for matching and capturing route strings.

pub use yew_router_route_parser::{
    parser::YewRouterParseError, Capture, CaptureVariant, Captures, FromCapturedKeyValue,
    FromCapturesError, MatcherToken,
};

pub use yew_router_route_parser::FromCaptures;

mod route_matcher;
pub use self::route_matcher::{MatcherSettings, RouteMatcher};
