
use yew::prelude::*;
use yew_router::prelude::*;
use yew::Properties;
use yew_router::router::FromPath;

pub struct CModel;

#[derive(PartialEq, Properties)]
pub struct Props{}

pub enum Msg {
}


impl Component for CModel {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        CModel
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}


impl Renderable<CModel> for CModel {
    fn view(&self) -> Html<Self> {

        html! {
            <div>
                {" I am the C component"}
            </div>
        }
    }
}

impl <T> FromPath<T> for Props {

    fn from_path(route: &RouteBase<T>) -> Option<Self> {
        let second_segment = route.path_segments.get(1)?;
        if "c" == second_segment.as_str() {
            Some(Props{})
        } else {
            None
        }
    }
}

//impl Routable for CModel {
//
//    fn resolve_props(route: &Route) -> Option<Self::Properties> {
//        let second_segment = route.path_segments.get(1).unwrap();
//        if "c" == second_segment.as_str() {
//            Some(Props)
//        } else {
//            None
//        }
//    }
//    fn will_try_to_route(route: &Route) -> bool {
//        route.path_segments.get(1).is_some()
//    }
//}

