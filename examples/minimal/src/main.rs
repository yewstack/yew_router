#![recursion_limit = "256"]
use yew::prelude::*;

use yew_router::{route::Route, service::RouteService, Switch};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::initialize();
    web_logger::init();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}

pub struct Model {
    route_service: RouteService<()>,
    route: Route<()>,
}

pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut route_service: RouteService<()> = RouteService::new();
        let route = route_service.get_route();
        let route = Route::from(route);
        let callback = link.send_back(|(route, state)| -> Msg {
            Msg::RouteChanged(Route {
                route,
                state: Some(state),
            })
        });
        route_service.register_callback(callback);
        Model {
            route_service,
            route,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => self.route = route,
            Msg::ChangeRoute(route) => {
                // This might be derived in the future
                let route_string = match route {
                    AppRoute::A(s) => format!("/a/{}", s),
                    AppRoute::B { anything, number } => format!("/b/{}/{}", anything, number),
                    AppRoute::C => format!("/c"),
                };
                self.route_service.set_route(&route_string, ());
                self.route = Route {
                    route: route_string,
                    state: None,
                };
            }
        }
        true
    }
}

#[derive(Debug, Switch)]
pub enum AppRoute {
    #[to = "/a/{anything}"]
    A(String),
    #[to = "/b/{anything}/{number}"]
    B { anything: String, number: u32 },
    #[to = "/c"]
    C,
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <nav class="menu",>
                    <button onclick=|_| Msg::ChangeRoute(AppRoute::A("lorem".to_string())) > {"A"} </button>
                    <button onclick=|_| Msg::ChangeRoute(AppRoute::B{anything: "hello".to_string(), number: 42}) > {"B"} </button>
                    <button onclick=|_| Msg::ChangeRoute(AppRoute::C) > {"C"} </button>
                </nav>
                <div>
                {
                    match AppRoute::switch(self.route.clone()) {
                        Some(AppRoute::A(thing)) => html!{thing},
                        Some(AppRoute::B{anything, number}) => html!{<div> {anything} {number} </div>},
                        Some(AppRoute::C) => html!{"C"},
                        None => html!{"404"}
                    }
                }
                </div>
            </div>
        }
    }
}
