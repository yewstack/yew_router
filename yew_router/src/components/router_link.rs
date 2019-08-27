use crate::route_info::RouteInfo;
use crate::route_agent::{RouteAgent, RouteRequest};
use yew::prelude::*;

use super::Msg;
use super::Props;

/// An anchor tag Component that when clicked, will navigate to the provided route.
/// The Route's `to_route_string()` will be displayed as the href.
pub struct RouterLink {
    router: Box<dyn Bridge<RouteAgent<()>>>,
    // TODO make this hold a link and a optional state instead, so they can each independently be passed in as props.
//    route: RouteInfo<()>,
    props: Props
//    link: String,
//    state: (),
//    text: String,
//    disabled: bool,
//    class: String,
}

impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_route: RouteInfo<()>| Msg::NoOp);
        let router = RouteAgent::bridge(callback);

        RouterLink {
            router,
            props
//            route: props.route,
//            text: props.text,
//            disabled: props.disabled,
//            class: props.class,
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
//        self.route = props.route;
//        self.text = props.text;
//        self.disabled = props.disabled;
//        self.class = props.class;
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
                class=self.props.class.clone(),
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
