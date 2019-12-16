//! Components that integrate with the [route agent](agent/struct.RouteAgent.html).
//!
//! At least one bridge to the agent needs to exist for these to work.
//! This can be done transitively by using a `Router` component, which owns a bridge to the agent.

mod router_button;
mod router_link;

use yew::{Children, Properties};

#[allow(deprecated)]
pub use self::{router_button::RouterButton, router_link::RouterAnchor, router_link::RouterLink};
use crate::RouterState;

// TODO This should also be PartialEq and Clone. Its blocked on Children not supporting that.
/// Properties for `RouterButton` and `RouterLink`.
#[derive(Properties, Default, Debug)]
pub struct Props<T: RouterState> {
    /// The route that will be set when the component is clicked.
    pub link: String,
    /// The state to set when changing the route.
    pub state: Option<T>,
    #[deprecated(note = "Use children field instead (nested html)")]
    /// The text to display.
    pub text: String,
    /// Html inside the component.
    pub children: Children,
    /// Disable the component.
    pub disabled: bool,
    /// Classes to be added to component.
    pub classes: String,
}

/// Message for `RouterButton` and `RouterLink`.
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    /// Tell the router to navigate the application to the Component's pre-defined route.
    Clicked,
}
