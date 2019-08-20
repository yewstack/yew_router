
use yew::prelude::*;
use yew_router::components::router_button::RouterButton;
use c_component::CModel;
use yew::Properties;
use yew_router::Router;

use yew_router::route::RouteInfo;
use yew_router::Route;
use c_component;
use yew_router::yew_router_derive::{FromMatches, route};
use yew_router::yew_router_route_parser::{PathMatcher, OptimizedToken};
use yew_router::Route2;

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
//                    <Router<()>: route_options=vec![
//                        Route::component::<CModel, _>(|route| c_component::Props::from_route_info(route)),
//                        Route2::new::<CModel>(route!("/{}/c"))
//                        Route::children(|_| html!{
//                            <div>
//                                {"404 page"}
//                            </div>
//                        })
//                    ], />
//                    <YewRouter: routes=routes![CModel], />
                </div>
            </div>
        }
    }
}

//impl Routable for AModel {
//    // Once proc macros land, it wouldn't be too difficult to set up a macro that does all of the below that looks like
//    // #[route("/a/<sub_path>#<number>")]
//    // That will implement this trait for the Component.
//    //
//    // The syntax could be extended to not care about prior paths like so:
//    // #[route("/*/<sub_path>#<number>")]
//    fn resolve_props(route: &Route) -> Option<Self::Properties> {
//        let first_segment = route.path_segments.get(0).unwrap();
//        if "a" == first_segment.as_str() {
//            Some(Props)
//        } else {
//            None // This will only render if the first path segment is "a"
//        }
//    }
//
//    fn will_try_to_route(route: &Route) -> bool {
//        route.path_segments.get(0).is_some()
//    }
//
//}



use yew_router::router::FromRouteInfo;
impl <T> FromRouteInfo<T> for Props {

    fn from_route_info(route: &RouteInfo<T>) -> Option<Self> {
        let first_segment = route.path_segments.get(0)?;
        if "a" == first_segment.as_str() {
            Some(Props{})
        } else {
            None // This will only render if the first path segment is "a"
        }

    }
}
