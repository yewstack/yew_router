extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate yew;

extern crate stdweb;
/// Possibly move the code for the directional message bus into this project
extern crate yew_patterns;

pub mod routing_service;

#[cfg(feature = "router_agent")]
pub mod router_agent;

#[cfg(feature = "router_agent")]
pub mod route;
#[cfg(feature = "router_agent")]
pub use route::SimpleRouteInfo;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "yew_router")]
mod component_router;
#[cfg(feature = "yew_router")]
pub use component_router::*;

// TODO preludes have kind of fallen out of favor, maybe this should be removed ???
pub mod prelude {
    //    #[cfg(feature = "yew_router")]
    //    pub use super::component_router::{YewRouter, Props, DefaultPage, RoutableBase, Routable};

    #[cfg(feature = "router_agent")]
    pub use super::route::{RouteInfo, SimpleRouteInfo};
    #[cfg(feature = "router_agent")]
    pub use super::router_agent::{Router, RouterRequest, SimpleRouterAgent};
}
