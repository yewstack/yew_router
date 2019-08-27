use crate::route_info::RouteInfo;
use crate::route_agent::{RouteAgent, RouteRequest};
use yew::prelude::*;

use super::Msg;
use super::Props;

pub struct RouterButton {
    router: Box<dyn Bridge<RouteAgent<()>>>,
    props: Props,
}

impl Component for RouterButton {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_route: RouteInfo<()>| Msg::NoOp);
        let router = RouteAgent::bridge(callback);

        RouterButton {
            router,
            props
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false,
            Msg::Clicked => {
                let route_info = RouteInfo {
                    route: self.props.link.clone(),
                    state: self.props.state.clone()
                };
                self.router
                    .send(RouteRequest::ChangeRoute(route_info));
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
                class=self.props.class.clone(),
                onclick=|_| Msg::Clicked,
                disabled=self.props.disabled,
            >
                {&self.props.text}
            </button>
        }
    }
}
