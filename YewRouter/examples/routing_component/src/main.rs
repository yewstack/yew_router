#![recursion_limit="256"]
extern crate yew;
extern crate yew_router;

mod b_component;
mod a_component;
mod c_component;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::{Router, Route};
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

        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), route=RouteInfo::from("/a/"), />
                    <RouterLink: text=String::from("Go to B"), route=RouteInfo::from("/b/#"), />
                    <RouterButton: text=String::from("Go to C"), route=RouteInfo::from("/c"), />
                    <RouterButton: text=String::from("Go to A/C"), route=RouteInfo::from("/a/c"), />
                </nav>
                <div>
                    <Router>
                        <Route path=route!("/a/{}" => AModel) />
                        <Route path=route!("/c" => CModel) />
                        <Route path=route!("/b/{sub_path}" => BModel) />
                    </Router>
                </div>
            </div>
        }
    }
}

