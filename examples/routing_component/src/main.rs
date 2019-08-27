#![recursion_limit="1024"]
mod b_component;
mod a_component;
mod c_component;

use yew::prelude::*;
use yew_router::{Router, Route};
use yew_router::components::RouterButton;
use yew_router::components::RouterLink;
use yew_router::RouteInfo;
use yew_router::route;

use crate::b_component::BModel;
use crate::a_component::AModel;
use crate::c_component::CModel;

use yew_router::render::component;
use yew_router::render::Render;
use yew_router::path_matcher::Matches;

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
                    <RouterButton: text=String::from("Go to bad path"), route=RouteInfo::from("/a_bad_path"), />
                </nav>
                <div>
                    <Router>
                        <Route path=route!("/a/{}") render=component::<AModel>() />
                        <Route path=route!("/c") render=component::<CModel>() />
                        <Route path=route!("/b(?sub_path={sub_path})(#{number})") render=component::<BModel>()/>
                        <Route path=route!("/e/c") render=component::<CModel>() >
                             {"Hello there from the other E \"child\""}
                        </Route>
                        <Route path=route!("/e")>
                             {"Hello there from the E \"child\""}
                        </Route>
                        <Route path=route!("/f/{capture}")
                            render=Render::new(|matches: &Matches| {
                                Some(html!{
                                    {format!("hello {}", matches[&"capture"])}
                                })
                            })
                        />
                        <Route path=route!("{*:any}") render=Render::new(|matches: &Matches| {
                            Some(html!{{format!("404, page not found for '{}'", matches["any"])}})
                        }) />
                    </Router>
                </div>
            </div>
        }
    }
}

