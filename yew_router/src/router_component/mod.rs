pub mod render;
pub mod route;
pub mod router;


use crate::agent::RouterState;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq {}

impl<'de, T> YewRouterState<'de> for T where T: RouterState<'de> + PartialEq {}
