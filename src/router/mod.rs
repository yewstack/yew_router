//! Router component and related types.
mod render;
mod route;
mod router;

pub use router::{Router, Props};
pub use route::{Route, RouteProps};
pub use render::{RenderFn, Render, render, component};
pub(crate) use render::{create_component_with_scope};

use crate::agent::AgentState;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait RouterState<'de>: AgentState<'de> + PartialEq {}

impl<'de, T> RouterState<'de> for T where T: AgentState<'de> + PartialEq {}
