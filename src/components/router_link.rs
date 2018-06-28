use yew::prelude::*;
use route::RouteBase;
use router_agent::{RouterAgentBase, RouterRequest};

use super::Props;


pub enum Msg {
    NoOp,
    Clicked
}

pub struct RouterLink {
    router: Box<Bridge<RouterAgentBase<()>>>,
    route: RouteBase<()>,
    text: String,
    disabled: bool,
    class: String
}


impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|_route: RouteBase<()>| Msg::NoOp);
        let router = RouterAgentBase::bridge(callback);

        RouterLink {
            router,
            route: props.route,
            text: props.text,
            disabled: props.disabled,
            class: props.class
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false,
            Msg::Clicked => {
                self.router.send(RouterRequest::ChangeRoute(self.route.clone()));
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

impl Renderable<RouterLink> for RouterLink {
    fn view(&self) -> Html<RouterLink> {
        let mut target = self.route.to_route_string();
        if !target.contains("#") {
            target = format!("{}#",target);
        }
        html! {
            <a
                class=&self.class,
                onclick=|_| Msg::Clicked,
                disabled=self.disabled,
                href=target,
            >
                {&self.text}
            </a>
        }
    }
}