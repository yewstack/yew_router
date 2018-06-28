#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate yew;

extern crate yew_patterns;

extern crate stdweb;


pub mod router;
mod routing_service;
pub mod components;
//pub mod yew_router;
mod component_routers;
pub use component_routers::*;
//pub use router::{Router, RouterRequest};
pub mod route;
pub use route::Route;


pub mod prelude {
    pub use super::component_routers::{YewRouter, Props, DefaultPage, RoutableBase, Routable};
    pub use super::route::Route;
    pub use super::router::{Router, RouterRequest};
}


