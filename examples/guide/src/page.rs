use yew::{prelude::*, virtual_dom::VNode};

pub struct Page;

#[derive(Properties, Clone)]
pub struct PageProps {
    #[props(required)]
    pub uri: String,
    #[props(required)]
    pub page_url: String,
    #[props(required)]
    pub title: String,
}

impl Component for Page {
    type Message = ();
    type Properties = PageProps;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Page
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self) -> VNode {
        unimplemented!()
    }
}
