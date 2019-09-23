use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

pub struct PageNotFound {}

pub enum Msg {}

impl Component for PageNotFound {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        PageNotFound {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
    fn destroy(&mut self) {
        log::info!("PageNotFound destroyed")
    }
}

impl Renderable<PageNotFound> for PageNotFound {
    fn view(&self) -> Html<Self> {
        html! {
            {"Page Not Found"}
        }
    }
}
