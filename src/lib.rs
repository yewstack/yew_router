#![feature(never_type)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate yew;

/// Possibly move the code for the directional message bus into this project
extern crate yew_patterns;
extern crate stdweb;

pub mod routing_service;

#[cfg(feature = "router_agent")]
pub mod router_agent;

#[cfg(feature = "router_agent")]
pub mod route;
#[cfg(feature = "router_agent")]
pub use route::Route;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "yew_router")]
mod component_router;
#[cfg(feature = "yew_router")]
pub use component_router::*;

pub mod prelude {
    #[cfg(feature = "yew_router")]
    pub use super::component_router::{YewRouter, Props, DefaultPage, RoutableBase, Routable};

    #[cfg(feature = "router_agent")]
    pub use super::route::{Route, RouteBase};
    #[cfg(feature = "router_agent")]
    pub use super::router_agent::{RouterAgent, RouterRequest, Router};
}


