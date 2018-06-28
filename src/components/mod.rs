
pub mod router_link;
pub mod router_button;

pub use self::router_link::RouterLink;
pub use self::router_button::RouterButton;

use route::RouteBase;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Props {
    pub route: RouteBase<()>,
    pub text: String,
    pub disabled: bool,
    pub class: String
}
