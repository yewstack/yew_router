#![recursion_limit = "128"]
//! Provides routing faculties for the Yew web framework.

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

#[cfg(feature = "matcher")]
pub use matcher::route_matcher as path_matcher;


#[cfg(any(feature = "matcher", feature= "router" ) )]
pub mod matcher;

#[cfg(feature = "matchers")]
pub use yew_router_derive::{route, FromMatches};
