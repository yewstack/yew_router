use yew::prelude::*;
use route::RouteBase;
use router::{Router, RouterRequest};


#[derive(Default, Clone, Debug, PartialEq)]
pub struct Props {
    pub route: RouteBase<()>,
    pub text: String
}

pub enum Msg {
    NoOp,
    Clicked
}

pub struct RouterLink {
    router: Box<Bridge<Router<()>>>,
    route: RouteBase<()>,
    text: String
}


impl Component for RouterLink {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|_route: RouteBase<()>| Msg::NoOp);
        let router = Router::bridge(callback);

        RouterLink {
            router,
            route: props.route,
            text: props.text
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
        true
    }
}

impl Renderable<RouterLink> for RouterLink {
    fn view(&self) -> Html<RouterLink> {
        html! {
            <a onclick=|_| Msg::Clicked, >{&self.text}</a>
        }
    }
}