#![recursion_limit = "128"]
//! Provides routing faculties for the Yew web framework.
//!
//! ## Contents
//! This crate consists of multiple types, some independently useful on their own,
//! that are used together to facilitate routing within the Yew framework.
//! Among them are:
//! * RouteService - Hooks into the History API and listens to `PopStateEvent`s to respond to users clicking the back/forwards buttons.
//! * RouteAgent - A singleton agent that owns a RouteService that provides an easy place for other components and agents to hook into it.
//! * Router - A component that can choose one of its nested child Routes to render based on the URL.
//! * Route - A component that supplies a matching condition and a render target to the Router.
//! * Matcher - An enum that determines if a URL will match a given route. A custom syntax for declaring these exists with the `route!()` macro, or you can use a Regex, or supply your own matcher.
//! * RouteButton & RouteLink - Wrapper components around buttons and anchor tags respectively that allow users to change the route.
//! * RouteInjector - A component that allows the injection of the current route into its nested children.
//!
//! ## State and Aliases
//! Because the History API allows you to store data along with a route string,
//! most types have at type parameter that allows you to specify which type is being stored.
//! As this behavior is uncommon, aliases using the unit type (`()`) are provided to remove the
//! need to specify the storage type you likely aren't using.
//!
//! If you want to store state using the history API it is recommended that you generate your own aliases using the `define_router_state` macro.
//! Give it a typename, and it will generate a module containing aliases and functions useful for routing.
//!
//!
//!
//! ## Orphaning
//! Currently it is possible to "orphan" components in Yew. This happens when a component doesn't display anymore,
//! but isn't entirely cleaned up, causing it to still exist and respond to messages sent to it by agents.
//! This can have negative effects on your program when enough of these accumulate, if for example,
//! one of the orphaned components makes fetch requests at part of its component lifecycle.
//!
//! This library is particularly good at causing this bug to appear.
//!
//! To avoid the orphaning problem, it is recommended to only have one top-level Router component in your project for the moment.
//! Use this Router to collect as much information as needed for any sub-routes, and pass that along to children components via Properties.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_qualifications
)]
#![allow(deprecated)] // TODO remove me once dispatchers lands
use proc_macro_hack::proc_macro_hack;

mod alias;
pub mod route_service;

#[cfg(feature = "router_agent")]
pub mod agent;

pub mod route_info;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "router")]
mod router_component;
#[cfg(feature = "router")]
pub use router_component::{
    render, route, router, YewRouterState,
};



// Use this alias to define a module containing type aliases.
define_router_state!(());
pub use router_state::*;

pub use alias::*;

#[cfg(any(feature = "matchers", feature= "router" ) )]
pub mod matcher;
#[cfg(any(feature = "matchers", feature= "router" ) )]
pub use matcher::{MatcherProvider, Matcher, Captures, FromCaptures, FromCapturesError};


#[cfg(feature = "matchers")]
pub use yew_router_macro::FromCaptures;

/// The route macro produces a Matcher which can be used to determine if a route string should cause
/// a section of html or component should render.
///
/// ### Exact Matching
/// At its simplest, the macro will accept a literal string containing a path, query, and fragment.
/// Not all parts are required, but the macro will fail if they are specified out of order.
/// If the route acquired from the browser matches this exactly, the route will match, and the
/// associated target will be rendered.
///
/// ### Particularities
/// The route macro is pretty strict in what it allows.
/// It will reject your matcher string under the following circumstances:
/// * If `//` is a possible valid match sequence (this has implications for where Optional sections may be used)
/// * There can only be one `?` character. It denotes the start of the query section. Subsequent queries must begin with `&`.
/// * Incomplete queries are not allowed. They must follow the form of `<?|&><literal>=<literal|any>`.
///
/// ### Any Matching
/// On top of just matching strings literally, Any sections, denoted by `{}` can be supplied to
/// match anything in that section. They must match one or more characters to be valid.
/// Captured values will be available as part of a `Captures` struct if the matching succeeds.
/// `Captures` is a type alias to `HashMap<&str, String>`.
/// If you want to specify that a captured section must be a number, you must capture the string,
/// and then attempt to parse it to your desired numeric type in `from_matches` or `render`.
///
/// Any sections can come in multiple forms:
///
/// * `{}` - Matches non-separator characters until the end of the route or until the next section of exact characters are matched.
/// * `{key}`- Matches anything, just as above, but stores the captured characters as a String inside a HashMap with the specified name acting as a key.
/// * `{*}` - Ignore all path separators (`/`), consuming characters until the end of the route or the next section is exactly matched.
/// * `{*:key}` - Matches as above, but stores the captured characters as a String inside a HashMap.
/// * `{4}` - Consume the specified number of path separators (`/`) before being allowed to match against a terminating set of characters.
/// * `{4:key} - Same as above, but stores the captured characters as a String inside a HashMap.
///
/// There is a rule to remember here:
/// * Because Any matchers use the subsequent exact section to terminate their search, no Any matchers cannot be next to each other.
///   * Optional matchers do not adequately separate Any matchers, because they may not match at all, leaving no exact section to separate the Any matchers.
///
/// ### Optional Matching
/// Optional matchers are denoted by `()` characters. They must contain at least one character.
/// They either match their contents, or not at all.
/// Optional matchers cannot opt out of parts of queries, although they can opt out of entire queries.
///
/// ### Parser Options
/// There are currently three options for the parser. One or more may be specified after the
/// matcher string (delimited by spaces).
/// * `Strict`
/// * `CaseInsensitive`
/// * `Incomplete`
///
/// By default, an optimizing step will insert an optional `/` after the path if doing so is valid.
/// If you want to turn this behavior off, add the word `Strict` after the provided string.
///
/// The matcher is case sensitive by default, but that can be disabled by specifying `CaseInsensitive` after the provided string.
///
/// The matcher will fail by default if the provided route string isn't completely matched by the matcher.
/// Specifying `Incomplete` will allow it to succeed, even if the matcher doesn't complete the whole route string.
///
///
/// # Examples
///
/// #### Exact match
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem/ipsum/dolor/sit?amet=consectetur&adipiscing=elit#sed");
/// assert!(matcher.match_route_string("/lorem/ipsum/dolor/sit?amet=consectetur&adipiscing=elit#sed").is_some());
/// ```
///
/// #### Capture
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem/ipsum/{value1}/{value2}");
/// let captures = matcher.match_route_string("/lorem/ipsum/dolor/sit").unwrap();
/// assert_eq!(captures["value1"], "dolor".to_string());
/// assert_eq!(captures["value2"], "sit".to_string());
/// ```
///
/// #### Match Many
/// ```
/// # use yew_router::route;
/// let matcher = route!("/lorem/ipsum/{*}");
/// assert!(matcher.match_route_string("/lorem/ipsum/dolor/sit").is_some());
/// assert!(matcher.match_route_string("/lorem/ipsum/").is_some());
///
/// let matcher = route!("/{*}/dolor/sit");
/// assert!(matcher.match_route_string("/lorem/ipsum/dolor/sit").is_some());
/// assert!(matcher.match_route_string("/lorem/dolor/sit").is_some());
/// assert!(matcher.match_route_string("/dolor/sit").is_none());
/// ```
///
/// #### Match Optional
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem(/ipsum)");
/// assert!(matcher.match_route_string("/lorem/ipsum").is_some());
/// assert!(matcher.match_route_string("/lorem").is_some());
///
///
/// let matcher = route!("/lorem(/ipsum{any})");
/// let captures = matcher.match_route_string("/lorem/ipsumdolorsit").unwrap();
/// assert_eq!(captures["any"], "dolorsit".to_string());
/// ```
///
/// #### Trailing Auto-Optional Slash
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem");
/// assert!(matcher.match_route_string("/lorem").is_some());
/// assert!(matcher.match_route_string("/lorem/").is_some());
///
/// let matcher = route!("/lorem" Strict);
/// assert!(matcher.match_route_string("/lorem").is_some());
/// assert!(matcher.match_route_string("/lorem/").is_none());
/// ```
///
/// #### Case Insensitivity
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem/ipsum" CaseInsensitive);
/// assert!(matcher.match_route_string("/loReM/IpSuM").is_some());
/// ```
///
/// #### Incomplete
/// ```
///# use yew_router::route;
/// let matcher = route!("/lorem" Incomplete);
/// assert!(matcher.match_route_string("/lorem/ipsum").is_some());
/// ```
///
#[cfg(feature = "matchers")]
#[proc_macro_hack]
pub use yew_router_macro::route;
