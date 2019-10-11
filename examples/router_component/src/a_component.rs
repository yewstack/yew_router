use crate::c_component::CModel;
use yew::prelude::*;
use yew::Properties;
use yew_router::prelude::*;
use crate::ARoute;

pub struct AModel {
    props: Props
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    pub route: ARoute
}

pub enum Msg {}

impl Component for AModel {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        AModel {props}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
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
                {
                    match self.props.route {
                        ARoute::C => html!{<CModel/>},
                        ARoute::None => html!{}
                    }
                }
                </div>
            </div>
        }
    }
}
