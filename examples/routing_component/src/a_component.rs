
use router;
use router::Route;
use yew::prelude::*;

use yew_router::Routable;
use yew_router::YewRouter;
use yew_router::components::router_button::RouterButton;
use c_component::CModel;

use yew_router::RoutableBase;


pub struct AModel {
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Props;

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
                        route=Route::parse("/a/c"),
                    />
                    <RouterButton:
                        text=String::from("Go to a/d (Component does not exist)"),
                        route=Route::parse("/a/d"),
                    />
                </div>
                <div>
                    <YewRouter: routes=routes![CModel], />
                </div>
            </div>
        }
    }
}

impl Routable for AModel {
    // Once proc macros land, it wouldn't be too difficult to set up a macro that does all of the below that looks like
    // #[route("/a/<sub_path>#<number>")]
    // That will implement this trait for the Component.
    //
    // The syntax could be extended to not care about prior paths like so:
    // #[route("/*/<sub_path>#<number>")]
    fn resolve_props(route: &router::Route) -> Option<Self::Properties> {
        let first_segment = route.path_segments.get(0).unwrap();
        if "a" == first_segment.as_str() {
            Some(Props)
        } else {
            None // This will only render if the first path segment is "a"
        }
    }

    fn will_try_to_route(route: &router::Route) -> bool {
        route.path_segments.get(0).is_some()
    }

}

