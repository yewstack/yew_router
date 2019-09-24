#![recursion_limit = "1024"]
mod a_component;
mod b_component;
mod c_component;

use yew::prelude::*;

//use yew_router::route;
//use yew_router::{Route, Router};
use yew_router::prelude::*;
use yew_router::components::RouterButton;
use yew_router::components::RouterLink;

use crate::a_component::AModel;
use crate::b_component::BModel;
use crate::c_component::CModel;

//use yew_router::component;
//use yew_router::matcher::Captures;
//use yew_router::render;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
                    <RouterButton: text=String::from("Go to A"), link="/A/", />
                    <RouterLink: text=String::from("Go to B"), link="/b/#", />
                    <RouterButton: text=String::from("Go to C"), link="/c", />
                    <RouterButton: text=String::from("Go to A/C"), link="/a/c", />
                    <RouterButton: text=String::from("Go to E"), link="/e", />
                    <RouterButton: text=String::from("Go to E/C"), link="/e/c", />
                    <RouterButton: text=String::from("Go to F (hello there)"), link="/f/there", />
                    <RouterButton: text=String::from("Go to F (hello world)"), link="/f/world", />
                    <RouterButton: text=String::from("Go to bad path"), link="/a_bad_path", />
                </nav>
                <div>
                    <Router>
                        <Route matcher=route!("/a/{}"  Strict CaseInsensitive) render=component::<AModel>() />
                        <Route matcher=route!("/c") render=component::<CModel>() />
                        <Route matcher=route!("/b(?sub_path={sub_path})(#{number})") render=component::<BModel>()/>
                        <Route matcher=route!("/e")>
                             {"Hello there from the E \"child\""}
                        </Route>
                        <Route matcher=route!("/e/c") render=component::<CModel>() >
                             {"Hello there from the other E \"child\""}
                        </Route>
                        <Route
                            matcher=route!("/f/{capture}")
                            render=render(|captures: &Captures| {
                                Some(html!{
                                    {format!("hello {}", captures[&"capture"])}
                                })
                            })
                        />
                        <Route matcher=route!("{*:any}") render=render(|captures: &Captures| {
                            Some(html!{{format!("404, page not found for '{}'", captures["any"])}})
                        }) />
                    </Router>
                </div>
            </div>
        }
    }
}
