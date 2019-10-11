#![recursion_limit = "1024"]
mod a_component;
mod b_component;
mod c_component;

use yew::prelude::*;

use yew_router::prelude::*;
use yew_router::Switch;

use crate::a_component::AModel;
use crate::b_component::BModel;
use crate::c_component::CModel;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::initialize();
    web_logger::init();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}

pub struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }
}

#[derive(Debug, Switch)]
pub enum AppRoute {
    #[to = "/a{*:inner}"]
    A(ARoute),
    #[to = "/b/[?sub_path={sub_path}][#{number}]"]
    B{sub_path: Option<String>, number: Option<usize>},
    #[to = "/c"]
    C,
    #[to = "/e/{string}"]
    E(String),
}

#[derive(Debug, Switch, PartialEq, Clone, Copy)]
pub enum ARoute {
    /// Match "/c" after "/a" ("/a/c")
    #[to = "/c"]
    C,
    // Because it is impossible to specify an Optional nested route:
    // Still accept the route, when matching, but consider it invalid.
    // This is effectively the same as wrapping the ARoute in Option, but doesn't run afoul of the current routing syntax.
    #[to = "{*}"]
    None
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), link="/a", />
                    <RouterLink: text=String::from("Go to B"), link="/b/#", />
                    <RouterButton: text=String::from("Go to C"), link="/c", />
                    <RouterButton: text=String::from("Go to A/C"), link="/a/c", />
                    <RouterButton: text=String::from("Go to E (hello there)"), link="/e/there", />
                    <RouterButton: text=String::from("Go to E (hello world)"), link="/e/world", />
                    <RouterButton: text=String::from("Go to bad path"), link="/a_bad_path", />
                </nav>
                <div>
                    <Router<AppRoute, ()>
                        render = Router::render(|switch: Option<&AppRoute>| {
                            match switch {
                                Some(AppRoute::A(route)) => html!{<AModel route = route />},
                                Some(AppRoute::B{sub_path, number}) => html!{<BModel sub_path=sub_path.clone(), number=number.clone()/>},
                                Some(AppRoute::C) => html!{<CModel />},
                                Some(AppRoute::E(string)) => html!{format!("hello {}", string)},
                                None => html!{"404"}
                            }
                        })
                    />
                </div>
            </div>
        }
    }
}
