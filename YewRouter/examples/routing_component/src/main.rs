#![recursion_limit="512"]
extern crate yew;
extern crate yew_router;

mod b_component;
mod a_component;
mod c_component;

use yew::prelude::*;
use yew_router::{Router, Route};
use yew_router::components::RouterButton;
use yew_router::components::RouterLink;
use yew_router::RouteInfo;
use yew_router::route;


use b_component::BModel;
use a_component::AModel;
use c_component::CModel;
use std::collections::HashMap;


fn main() {
    yew::initialize();
    web_logger::init();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}


pub struct Model {}



impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {

        let f = |matches: &HashMap<&str, String>| {
            Some(html!{
                {format!("hello {}", matches[&"capture"])}
            })
        };

        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), route=RouteInfo::from("/a/"), />
                    <RouterLink: text=String::from("Go to B"), route=RouteInfo::from("/b/#"), />
                    <RouterButton: text=String::from("Go to C"), route=RouteInfo::from("/c"), />
                    <RouterButton: text=String::from("Go to A/C"), route=RouteInfo::from("/a/c"), />
                    <RouterButton: text=String::from("Go to E"), route=RouteInfo::from("/e"), />
                    <RouterButton: text=String::from("Go to E/C"), route=RouteInfo::from("/e/c"), />
                    <RouterButton: text=String::from("Go to F (hello there)"), route=RouteInfo::from("/f/there"), />
                    <RouterButton: text=String::from("Go to F (hello world)"), route=RouteInfo::from("/f/world"), />
                </nav>
                <div>
                    <Router>
                        <Route path=route!("/a/{}" => AModel) />
                        <Route path=route!("/c" => CModel) />
                        <Route path=route!("/b/{sub_path}" => BModel) />
                        <Route path=route!("/e/c" => CModel)>
                             {"Hello there from the other E \"child\""}
                        </Route>
                        <Route path=route!("/e")>
                             {"Hello there from the E \"child\""}
                        </Route>
                        <Route path=route!("/f/{capture}", f)/>
                    </Router>
                </div>
            </div>
        }
    }
}

