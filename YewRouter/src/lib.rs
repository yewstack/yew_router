#![recursion_limit="128"]

pub mod route_service;

#[cfg(feature = "router_agent")]
pub mod route_agent;
#[cfg(feature = "router_agent")]
/// Alias to [RouteAgent<()>](struct.RouteAgent.html).
pub type RouteAgent = route_agent::RouteAgent<()>;

#[cfg(feature = "router_agent")]
pub mod route_info;
#[cfg(feature = "router_agent")]
/// Alias to [RouteInfo<()>](struct.RouteInfo.html).
pub type RouteInfo = route_info::RouteInfo<()>;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "yew_router")]
mod component_router;
#[cfg(feature = "yew_router")]
pub use component_router::{router, route, Route, Router, YewRouterState};

#[cfg(feature = "yew_router")]
pub use yew_router_path_matcher as path_matcher;

#[cfg(feature = "yew_router")]
pub use yew_router_derive::{route, FromMatches};

