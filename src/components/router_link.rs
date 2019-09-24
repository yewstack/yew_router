//! A component wrapping an `<a>` tag that changes the route.
use crate::agent::{RouteRequest, RouteSenderBridge, Void};
use crate::route_info::{RouteInfo, RouteString};
use yew::prelude::*;

use super::Msg;
use super::Props;

/// An anchor tag Component that when clicked, will navigate to the provided route.
#[derive(Debug)]
pub struct RouterLink {
    router: RouteSenderBridge,
    props: Props,
}

impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_: Void| Msg::NoOp);
        let router = RouteSenderBridge::new(callback);
        RouterLink { router, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false,
            Msg::Clicked => {
                let route_info = RouteInfo {
                    route: RouteString::Unstructured(self.props.link.clone()),
                    state: self.props.state,
                };
                self.router.send(RouteRequest::ChangeRoute(route_info));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl Renderable<RouterLink> for RouterLink {
    fn view(&self) -> Html<RouterLink> {
        use stdweb::web::event::IEvent;
        let target: &str = &self.props.link;

        html! {
            <a
                class=self.props.classes.clone(),
                onclick=|event | {
                    event.prevent_default();
                    Msg::Clicked
                },
                disabled=self.props.disabled,
                href=target,
            >
                {&self.props.text}
            </a>
        }
    }
}
