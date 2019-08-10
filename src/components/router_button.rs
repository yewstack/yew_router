use route::RouteInfo;
use router_agent::{RouterAgent, RouterRequest};
use yew::prelude::*;

use super::Msg;
use super::Props;

pub struct RouterButton {
    router: Box<dyn Bridge<RouterAgent<()>>>,
    route: RouteInfo<()>,
    text: String,
    disabled: bool,
    class: String,
}

impl Component for RouterButton {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_route: RouteInfo<()>| Msg::NoOp);
        let router = RouterAgent::bridge(callback);

        RouterButton {
            router,
            route: props.route,
            text: props.text,
            disabled: props.disabled,
            class: props.class,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false,
            Msg::Clicked => {
                self.router
                    .send(RouterRequest::ChangeRoute(self.route.clone()));
                false
            }
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.route = props.route;
        self.text = props.text;
        self.disabled = props.disabled;
        self.class = props.class;
        true
    }
}

impl Renderable<RouterButton> for RouterButton {
    fn view(&self) -> Html<RouterButton> {
        html! {
            <button
                class=&self.class,
                onclick=|_| Msg::Clicked,
                disabled=self.disabled,
            >
                {&self.text}
            </button>
        }
    }
}
