use yew::prelude::*;
use route::RouteInfo;
use router_agent::{RouterAgent, RouterRequest};

use super::Props;
use super::Msg;



/// An anchor tag Component that when clicked, will navigate to the provided route.
/// The Route's `to_route_string()` will be displayed as the href.
pub struct RouterLink {
    router: Box<dyn Bridge<RouterAgent<()>>>,
    route: RouteInfo<()>,
    text: String,
    disabled: bool,
    class: String
}


impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|_route: RouteInfo<()>| Msg::NoOp);
        let router = RouterAgent::bridge(callback);

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
        use stdweb::web::event::IEvent;
        let target = self.route.to_route_string();

        html! {
            <a
                class=&self.class,
                onclick=|event | {
                    event.prevent_default();
                    Msg::Clicked
                },
                disabled=self.disabled,
                href=target,
            >
                {&self.text}
            </a>
        }
    }
}