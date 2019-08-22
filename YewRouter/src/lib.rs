#![recursion_limit="128"]
/// Possibly move the code for the directional message bus into this project
//extern crate yew_patterns;

pub mod route_service;

#[cfg(feature = "router_agent")]
pub mod route_agent;
#[cfg(feature = "router_agent")]
pub type RouterAgent = route_agent::RouteAgent<()>;

#[cfg(feature = "router_agent")]
pub mod route_info;
#[cfg(feature = "router_agent")]
pub type RouteInfo = route_info::RouteInfo<()>;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "yew_router")]
mod component_router;
#[cfg(feature = "yew_router")]
pub use component_router::{router, Route, Router, YewRouterState};

#[cfg(feature = "yew_router")]
pub use yew_router_path_matcher as path_matcher;

#[cfg(feature = "yew_router")]
pub use yew_router_derive::{route, FromMatches};

