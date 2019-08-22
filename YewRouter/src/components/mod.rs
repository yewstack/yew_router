//! Components that integrate with the router agent.
//!
pub mod router_button;
pub mod router_link;

pub use self::router_button::RouterButton;
pub use self::router_link::RouterLink;

use crate::route_info::RouteInfo;
use yew::Properties;

/// Properties for Routing Components
#[derive(Properties, Default, Clone, Debug, PartialEq)]
pub struct Props {
    // TODO make this hold a link and a optional state instead, so they can each independently be passed in as props.
    pub route: RouteInfo<()>,
    pub text: String,
    pub disabled: bool,
    pub class: String,
}

/// Message for Routing Components.
pub enum Msg {
    /// Perform no action
    NoOp,
    /// Tell the router to navigate the application to the Component's pre-defined route.
    Clicked,
}
