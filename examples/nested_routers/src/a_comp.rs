use yew::prelude::*;
use yew_router::prelude::*;

use crate::page_not_found::PageNotFound;

pub struct AComp {}

pub enum Msg {}

impl Component for AComp {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        AComp {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn destroy(&mut self) {
        log::info!("AComp destroyed")
    }
}

impl Renderable<AComp> for AComp {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <div>
                    { "I am the A component"}
                </div>
                <div>
                    <Router>
                        <Route matcher=route!("/a/{*}") render=component::<PageNotFound>() />
                    </Router>
                </div>
            </>
        }
    }
}
