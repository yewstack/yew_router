
use yew::prelude::*;
use yew_router::components::router_button::RouterButton;
use crate::c_component::CModel;
use yew::Properties;
use yew_router::route_info::RouteInfo;
use yew_router::{FromMatches};
use yew_router::{Router, Route};
use yew_router::route;
use yew_router::render::component;

pub struct AModel {
}

#[derive(PartialEq, Properties, FromMatches)]
pub struct Props{}

pub enum Msg {
}


impl Component for AModel {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {

        AModel {
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}


impl Renderable<AModel> for AModel {
    fn view(&self) -> Html<Self> {

        html! {
            <div>
                { "I am the A component"}
                <div>
                    <RouterButton:
                        text=String::from("Go to a/c"),
                        route=RouteInfo::from("/a/c"),
                    />
                    <RouterButton:
                        text=String::from("Go to a/d (Component does not exist)"),
                        route=RouteInfo::from("/a/d"),
                    />
                </div>
                <div>
                    <Router>
                        <Route path=route!("/{}/c") render=component::<CModel>() />
                    </Router>
                </div>
            </div>
        }
    }
}

