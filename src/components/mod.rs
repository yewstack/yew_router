/// Components that integrate with the router agent.

pub mod router_link;
pub mod router_button;

pub use self::router_link::RouterLink;
pub use self::router_button::RouterButton;

use route::RouteBase;
use yew::Properties;


/// Properties for Routing Components
#[derive(Properties, Default, Clone, Debug, PartialEq)]
pub struct Props {
    pub route: RouteBase<()>,
    pub text: String,
    pub disabled: bool,
    pub class: String
}

/// Message for Routing Components.
pub enum Msg {
    /// Perform no action
    NoOp,
    /// Tell the router to navigate the application to the Component's pre-defined route.
    Clicked
}
