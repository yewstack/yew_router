//! A component wrapping a `<button>` tag that changes the route.
use crate::agent::{RouteRequest, RouteSenderBridge, Void};
use crate::route_info::{RouteInfo, RouteString};
use yew::prelude::*;

use super::Msg;
use super::Props;

/// Changes the route when clicked.
#[derive(Debug)]
pub struct RouterButton {
    router: RouteSenderBridge,
    props: Props,
}

impl Component for RouterButton {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_: Void| Msg::NoOp);
        let router = RouteSenderBridge::new(callback);

        RouterButton { router, props }
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

impl Renderable<RouterButton> for RouterButton {
    fn view(&self) -> Html<RouterButton> {
        html! {
            <button
                class=self.props.classes.clone(),
                onclick=|_| Msg::Clicked,
                disabled=self.props.disabled,
            >
                {&self.props.text}
            </button>
        }
    }
}
