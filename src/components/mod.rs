//! Components that integrate with the [route agent](struct.RouteAgent.html).

mod router_button;
mod router_link;

use yew::Properties;

pub use self::router_button::RouterButton;
pub use self::router_link::RouterLink;
use crate::RouterState;

/// Properties for `RouterButton` and `RouterLink`.
#[derive(Properties, Default, Clone, Debug, PartialEq)]
pub struct Props<T: for<'de> RouterState<'de>> {
    /// The route that will be set when the component is clicked.
    pub link: String,
    /// The state to set when changing the route.
    pub state: Option<T>,
    /// The text to display.
    pub text: String,
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
