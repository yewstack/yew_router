#[macro_use]
extern crate yew;
extern crate yew_router;

mod b_component;

use yew::prelude::*;
use yew_router::router::{self, Route};
use yew_router::{YewRouter, Routable, DefaultPage};
use b_component::BModel;


fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}


pub enum Child {
    A,
    B,
}

pub struct Model {
    router: Box<Bridge<router::Router<()>>>
}

pub enum Msg {
    NavigateTo(Child),
    NoOp
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {

        // This component wont handle changes to the route, although it could if it wanted to.
        let callback = link.send_back(|_route: Route<()>| Msg::NoOp);
        let router = router::Router::bridge(callback);


        Model {
            router
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NavigateTo(child) => {

                let path_segments = match child {
                    Child::A => vec!["a".into()],
                    Child::B => vec!["b".into()],
                };

                let route = router::Route {
                    path_segments,
                    query: None,
                    fragment: None,
                    state: (),
                };

                self.router.send(router::Request::ChangeRoute(route));
                false
            }
            Msg::NoOp => false
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        // Note: it should be possible to prevent the app from resolving some routes based on
        // app state by not including some routes in this variable.
        // This would come in handy in preventing access to admin panels for unauthorized users
        // or providing different components for users who aren't logged in.
        let props: yew_router::Props = yew_router::Props {
            routes: vec![BModel::RESOLVER],
            routing_failed_page: Some(DefaultPage(routing_failed_page))
        };

        html! {
            <div>
                <nav class="menu",>
                    <button onclick=|_| Msg::NavigateTo(Child::A),>{ "Go to A" }</button>
                    <button onclick=|_| Msg::NavigateTo(Child::B),>{ "Go to B" }</button>
                </nav>
                <div>
                    <YewRouter: with props, />
                </div>
            </div>
        }
    }
}

fn routing_failed_page(route: &Route<()>) -> Html<YewRouter> {
    html! {
        <>
            {format!("Could not route: '{}'", route.to_route_string())}
        </>
    }
}
