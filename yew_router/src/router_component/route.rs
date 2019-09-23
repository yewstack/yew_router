//! Route Component.
use super::YewRouterState;
use crate::router_component::render::Render;
use crate::router_component::router::Router;
use std::fmt::{Debug, Error as FmtError, Formatter};
use yew::{Children, Component, ComponentLink, Properties, ShouldRender};
use crate::matcher::Matcher;

/// A nested component used inside of [Router](../router/struct.Router.html) that can determine if a
/// sub-component can be rendered.
#[derive(Debug)]
pub struct Route<T: for<'de> YewRouterState<'de>> {
    props: RouteProps<T>,
}

/// Properties for Route.
///
/// The path matcher must be specified.
///
/// If only a `render` is specified, it will display its contents if it returns `Some` after the
/// path matcher succeeds in matching the URL.
/// If only the `children` are specified, they will be rendered if the path matcher matches the URL.
/// If both the `render` and `children` are specified, they will only both render
/// (`render` elements above the `children` elements in the DOM)
/// if the `render` returns `Some`.
#[derive(Properties)]
pub struct RouteProps<T: for<'de> YewRouterState<'de>> {
    /// Matches the url and can extract sections as matches to be used by the `Render`.
    #[props(required)]
    pub matcher: Matcher,
    /// Given matches matched from the URL, conditionally render the elements specified within.
    pub render: Render<T>,
    /// Will be rendered if it contains anything provided the `PathMatcher` matches the URL.
    pub children: Children<Router<T>>,
}

impl<T: for<'de> YewRouterState<'de>> Debug for RouteProps<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("RouteProps")
            .field("matcher", &self.matcher)
            .field("render", &self.render)
            .field("children (length)", &self.children.len())
            .finish()
    }
}

impl<T: for<'de> YewRouterState<'de>> Component for Route<T> {
    type Message = ();
    type Properties = RouteProps<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Route { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}
