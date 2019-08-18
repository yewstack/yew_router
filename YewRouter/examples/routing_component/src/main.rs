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
use yew_router::Route;


use b_component::BModel;
use a_component::AModel;


fn main() {
    yew::initialize();
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
                    <RouterButton: text=String::from("Go to A"), route=RouteInfo::parse("/a"), />
                    <RouterLink: text=String::from("Go to B"), route=RouteInfo::parse("/b"), />
                    <RouterButton: text=String::from("Go to A/C"), route=RouteInfo::parse("/a/c"), />
                </nav>
                <div>
                    <Router<()>:
                        route_options = vec![
                            Route::component_from_route_info::<AModel>(),
                            Route::component_from_route_info::<BModel>(),
                            Route::children(|_| html!{
                                <div>
                                    {"404 page"}
                                </div>
                            })
                        ],
                    />
                </div>
            </div>
        }
    }
}

