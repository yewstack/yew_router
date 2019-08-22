#![recursion_limit="256"]
extern crate yew;
extern crate yew_router;

mod b_component;
mod a_component;
mod c_component;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Router;
use yew_router::components::RouterButton;
use yew_router::components::RouterLink;


use b_component::BModel;
use a_component::AModel;
use c_component::CModel;

use yew_router::yew_router_derive::route;


fn main() {
    yew::initialize();
    web_logger::init();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}



pub struct Model {
}

pub enum Msg {
    NoOp
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NoOp => false
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        use yew_router::router::RouteChild;

        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), route=RouteInfo::parse("/a/"), />
                    <RouterLink: text=String::from("Go to B"), route=RouteInfo::parse("/b/#"), />
                    <RouterButton: text=String::from("Go to C"), route=RouteInfo::parse("/c"), />
                    <RouterButton: text=String::from("Go to A/C"), route=RouteInfo::parse("/a/c"), />
                </nav>
                <div>
                    <Router>
                        <RouteChild path=route!("/a/{}" => AModel) />
                        <RouteChild path=route!("/c" => CModel) />
                        <RouteChild path=route!("/b/{sub_path}" => BModel) />
                    </Router>
                </div>
            </div>
        }
    }
}

