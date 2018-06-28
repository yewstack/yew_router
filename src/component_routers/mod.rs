pub mod yew_router;
mod routable;

pub use self::yew_router::{YewRouter, DefaultPage, Props};
pub use self::routable::{Routable, RoutableBase};

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
