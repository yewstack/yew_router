pub mod router;

pub use self::router::{Props, Router};

use crate::router_agent::RouterState;

/// Any state that can be managed by the `YewRouter` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq {}

impl<'de, T> YewRouterState<'de> for T where T: RouterState<'de> + PartialEq {}

