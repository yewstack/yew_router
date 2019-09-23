use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew_router::{Router, Route, route, component};
use yew_router::components::RouterButton;
use crate::page_not_found::PageNotFound;
use crate::a_comp::AComp;
use crate::b_comp::BComp;


mod page_not_found;
mod a_comp;
mod b_comp;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


fn main() {
    web_logger::init();
    yew::start_app::<Model>();
}

struct Model { }

enum Msg {
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { }
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

