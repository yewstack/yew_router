extern crate yew;
extern crate yew_router;

mod b_component;
mod a_component;
mod c_component;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Router;
use yew_router::Props as RouterProps;
use yew_router::components::RouterButton;
use yew_router::components::RouterLink;
use yew_router::RouterOption;
use std::convert::TryFrom;

//use yew_router::{YewRouter, Route,  RoutableBase, DefaultPage};

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

        // This component wont handle changes to the route, although it could if it wanted to.
//        let callback = link.send_back(|_route: Route<()>| Msg::NoOp);
//        let router = router::Router::bridge(callback);


        Model {
//            router
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
        let router_props = RouterProps {
            route_options: vec![
                RouterOption::new::<AModel, _>(|route| a_component::Props::try_from(route).ok()),
                RouterOption::new::<BModel, _>(|route| b_component::Props::try_from(route).ok())
            ]
        };
        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), route=Route::parse("/a"), />
                    <RouterLink: text=String::from("Go to B"), route=Route::parse("/b"), />
                    <RouterButton: text=String::from("Go to A/C"), route=Route::parse("/a/c"), />
                </nav>
                <div>
                    <Router<()>: with router_props, />
                </div>
            </div>
        }
    }
}

