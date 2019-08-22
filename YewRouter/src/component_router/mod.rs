
pub mod router;


/// Alias to [Router<()>](struct.Router.html)
///
/// # Note
/// Because most end users will not use the ability to store state,
/// this alias is used to make the most common type of utilization of the router easier to type and read.
pub type Router = router::Router<()>;

/// Alias to [Route<()>](struct.Route.html)
///
/// # Note
/// Because most end users will not use the ability to store state,
/// this alias is used to make the most common type of utilization of route easier to type and read.
pub type Route = router::Route<()>;


use crate::router_agent::RouterState;

/// Any state that can be managed by the `YewRouter` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq {}

impl<'de, T> YewRouterState<'de> for T where T: RouterState<'de> + PartialEq {}

