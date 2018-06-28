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
mod routing;
pub mod components;
//pub mod yew_router;
mod component_routers;
pub use component_routers::*;
pub use router::Router;
