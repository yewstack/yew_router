
use yew::prelude::*;
use yew_router::components::router_button::RouterButton;
use c_component::CModel;
use yew::Properties;
use yew_router::Router;

use yew_router::route::RouteInfo;
use yew_router::yew_router_derive::{FromMatches, route};
use yew_router::router::RouteChild;

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
                        route=RouteInfo::parse("/a/c"),
                    />
                    <RouterButton:
                        text=String::from("Go to a/d (Component does not exist)"),
                        route=RouteInfo::parse("/a/d"),
                    />
                </div>
                <div>
                    <Router>
                        <RouteChild path=route!("/{}/c" => CModel) />
                    </Router>
                </div>
            </div>
        }
    }
}

