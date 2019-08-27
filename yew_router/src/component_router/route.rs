//! Route Component.
use yew_router_path_matcher::{PathMatcher};
use yew::{Component, ComponentLink, ShouldRender, Properties, Children};
use super::YewRouterState;
use crate::component_router::router::Router;
use crate::component_router::render::Render;

/// A nested component used inside of [Router](struct.Router.html) that can determine if a
/// sub-component can be rendered.
pub struct Route<T: for<'de> YewRouterState<'de>> {
    props: RouteProps<T>
}



/// Properties for Route.
#[derive(Properties)]
pub struct RouteProps<T: for<'de> YewRouterState<'de>> {
    #[props(required)]
    pub path: PathMatcher,
    pub render: Render<T>,
    pub children: Children<Router<T>>
}

impl <T: for<'de> YewRouterState<'de>> Component for Route<T> {
    type Message = ();
    type Properties = RouteProps<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Route {
            props
        }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

