use crate::a_comp::AComp;
use crate::b_comp::BComp;
use crate::page_not_found::PageNotFound;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew_router::components::RouterButton;
use yew_router::{route};
use yew_router::prelude::*;


mod a_comp;
mod b_comp;
mod page_not_found;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}

struct Model {}

enum Msg {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <RouterButton text="A" link="/a/" />
                <RouterButton text="B" link="/b/" />

                <Router>
                    <Route matcher=route!("/a") render=component::<AComp>() />
                    <Route matcher=route!("/b") render=component::<BComp>() />
                    <Route matcher=route!("{*}") render=component::<PageNotFound>() />
                </Router>
            </>
        }
    }
}
