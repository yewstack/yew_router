pub mod router;

//pub use self::yew_router::{YewRouter, DefaultPage, Props};
//pub use self::routable::{Routable, RoutableBase};
pub use self::router::{Props, Route, Router, Route2};

use crate::router_agent::RouterState;

/// Any state that can be managed by the `YewRouter` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq {}

impl<'de, T> YewRouterState<'de> for T where T: RouterState<'de> + PartialEq {}

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
