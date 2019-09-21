use crate::c_component::CModel;
use yew::prelude::*;
use yew::Properties;
use yew_router::components::router_button::RouterButton;
use yew_router::render::component;
use yew_router::route;
use yew_router::FromCaptures;
use yew_router::{Route, Router};

pub struct AModel {}

#[derive(PartialEq, Properties, FromCaptures)]
pub struct Props {}

pub enum Msg {}

impl Component for AModel {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        AModel {}
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
                        link="/a/c",
                    />
                    <RouterButton:
                        text=String::from("Go to a/d (Component does not exist)"),
                        link="/a/d",
                    />
                </div>
                <div>
                    <Router>
                        <Route matcher=route!("/{}/c") render=component::<CModel>() />
                    </Router>
                </div>
            </div>
        }
    }
}
