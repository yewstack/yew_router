pub mod render;
pub mod route;
pub mod router;


use crate::agent::RouterState;

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
pub type Route = route::Route<()>;

/// Alias to [Render<()>](struct.Render.html)
///
/// # Note
/// Because most end users will not use the ability to store state,
/// this alias is used to make the most common type of utilization of render function wrappers easier to type and read.
pub type Render = render::Render<()>;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait YewRouterState<'de>: RouterState<'de> + PartialEq {}

impl<'de, T> YewRouterState<'de> for T where T: RouterState<'de> + PartialEq {}
