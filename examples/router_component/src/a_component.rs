use crate::{c_component::CModel, ARoute};
use yew::{prelude::*, virtual_dom::VNode, Properties};
use yew_router::prelude::*;

pub struct AModel {
    props: Props,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    pub route: Option<ARoute>,
}

pub enum Msg {}

impl Component for AModel {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        AModel { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> VNode<Self> {
        html! {
            <div>
                { "I am the A component"}
                <div>
                    <RouterButton:
                        text=String::from("Go to a/c"),
                        link="/a/c",
                    />
                    <RouterButton:
                        text=String::from("Go to a/d (route does not exist)"),
                        link="/a/d",
                    />
                </div>
                <div>
                {
                    match self.props.route {
                        Some(_) => html!{<CModel/>},
                        None => html!{}
                    }
                }
                </div>
            </div>
        }
    }
}
