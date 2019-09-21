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
//! If you do indeed want to specify the data to be stored in the History API consider the following:
//! * The aliased types are typically named the same as their implementations, but are importable from a module higher up the hierarchy. You will have to look around for them in the docs as they aren't reexported at the highest level like their respective aliases.
//! * You should use the same state type parameter everywhere. Having different state types means multiple RouteAgents will be spawned and they will not communicate with routers of differing state types.
//! * You probably want to wrap your type in an `Option` and alias your type to make specifying it easier.
//! * There are varying, mostly undocumented, limits to the maximum size of objects you can store in the History API across different browsers (firefox appears to be the lowest bar at 640kb). Keeping this in mind for cross-browser compatibility is a must.
//! * If you are building a large application, it is a good idea to alias all entities used from this crate to use your specific state type much like has already been done with `()`.
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
use proc_macro_hack::proc_macro_hack;

pub mod route_service;

#[cfg(feature = "router_agent")]
pub mod route_agent;
#[cfg(feature = "router_agent")]
/// Alias to [RouteAgent<()>](struct.RouteAgent.html).
pub type RouteAgent = route_agent::RouteAgent<()>;

pub mod route_info;
/// Alias to [RouteInfo<()>](struct.RouteInfo.html).
pub type RouteInfo = route_info::RouteInfo<()>;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "router")]
mod router_component;
#[cfg(feature = "router")]
pub use router_component::{
    render, render::component, route, router, Render, Route, Router, YewRouterState,
};



#[cfg(any(feature = "matchers", feature= "router" ) )]
pub mod matcher;

#[cfg(feature = "matchers")]
pub use yew_router_macro::FromMatches;

/// The route macro produces a Matcher which can be used to determine if a route string should cause
/// a section of html or component should render.
///
/// At its simplest, the macro will accept a literal string containing a path, query, and fragment.
/// Not all parts are required, but the macro will fail if they are specified out of order.
/// If the route acquired from the browser matches this exactly, the route will match, and the
/// associated target will be rendered.
///
///
/// On top of just matching strings literally, match sections, denoted by `{}` can be supplied to
/// match anything in that section.
///
/// If a key starting with a valid rust identifier is supplied between the brackets like
/// `{key}` then the characters that gets matched by this section will be captured and will become
/// available in a HashMap returned from the matcher if it succeeds.
///
/// If a `*` is contained within the brackets like `{*}`, then the capture section will match across all paths.
///
/// If a number is specified within the brackets like `{3}`, then that number of path separators (`/`)
/// must be encountered before that match rule ends. `{1}` is equivalent to `{}`.
///
/// These `*` and number variants can be combined with the key-capture feature to capture sections to
/// apply these rules while capturing sections with a named key. This looks like `{*:key}` or `{2:other_key}`.
#[cfg(feature = "matchers")]
#[proc_macro_hack]
pub use yew_router_macro::route;
