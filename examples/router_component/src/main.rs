#![recursion_limit = "1024"]
mod a_component;
mod b_component;
mod c_component;

use yew::prelude::*;

use yew_router::{prelude::*, Switch};

use crate::{
    a_component::AModel,
    b_component::{BModel, BRoute},
    c_component::CModel,
};
use yew_router::switch::AllowMissing;
use yew::virtual_dom::VNode;

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

    fn view(&self) -> VNode<Self> {
        html! {
            <div>
                <nav class="menu",>
                    <RouterButton: text=String::from("Go to A"), link="/a/", />
                    <RouterLink: text=String::from("Go to B"), link="/b/#", />
                    <RouterButton: text=String::from("Go to C"), link="/c", />
                    <RouterButton: text=String::from("Go to A/C"), link="/a/c", />
                    <RouterButton: text=String::from("Go to E (hello there)"), link="/e/there", />
                    <RouterButton: text=String::from("Go to E (hello world)"), link="/e/world", />
                    <RouterButton: text=String::from("Go to bad path"), link="/a_bad_path", />
                </nav>
                <div>
                    <Router<AppRoute, ()>
                        render = Router::render(|switch: AppRoute| {
                            match switch {
                                AppRoute::A(AllowMissing(route)) => html!{<AModel route = route />},
                                AppRoute::B(route) => {
                                    let route: b_component::Props = route.into();
                                    html!{<BModel with route/>}
                                },
                                AppRoute::C => html!{<CModel />},
                                AppRoute::E(string) => html!{format!("hello {}", string)},
                                AppRoute::PageNotFound(None) => html!{"Page not found"},
                                AppRoute::PageNotFound(Some(missed_route)) => html!{format!("Page '{}' not found", missed_route)}
                            }
                        })
                        redirect = Router::redirect(|route: Route| {
                            AppRoute::PageNotFound(Some(route.route))
                        })
                    />
                </div>
            </div>
        }
    }
}

#[derive(Debug, Switch)]
pub enum AppRoute {
    #[to = "/a{*:inner}"]
    A(AllowMissing<ARoute>),
    #[to = "/b{*:inner}"]
    B(BRoute),
    #[to = "/c"]
    C,
    #[to = "/e/{string}"]
    E(String),
    #[to = "/page-not-found"]
    PageNotFound(Option<String>),
}

#[derive(Debug, Switch, PartialEq, Clone)]
#[to = "/c"]
pub struct ARoute;
