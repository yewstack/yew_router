//pub mod yew_router;
//mod routable;
//pub mod router2;
pub mod router3;

//pub use self::yew_router::{YewRouter, DefaultPage, Props};
//pub use self::routable::{Routable, RoutableBase};
pub use self::router3::{Router, Props, RouterOption};


use router_agent::RouterState;
use yew::Properties;

/// Any state that can be managed by the `YewRouter` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq + Properties {}

impl <'de, T> YewRouterState<'de> for T
    where T: RouterState<'de> + PartialEq + Properties
{}

/// Turns the provided component type name into wrapped functions that will create the component.
#[macro_export]
macro_rules! routes {
    ( $( $x:tt ), * ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                let v = $x::RESOLVER;
                temp_vec.push(v);
            )*
            temp_vec
        }
    };
}
