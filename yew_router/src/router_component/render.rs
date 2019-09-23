//! Wrapper around RenderFn that allows clones.
use crate::router_component::YewRouterState;
use crate::router::Router;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::rc::Rc;
use yew::virtual_dom::vcomp::ScopeHolder;
use yew::virtual_dom::{VComp, VNode};
use yew::{Component, Html, Renderable};
use crate::matcher::{FromCaptures, Captures, RenderFn};

/// Creates a component using supplied props.
fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default();
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}

/// Creates a `Render` that creates the specified component if its
/// props can be created from the provided matches using `FromCaptures`.
pub fn component<T, U>() -> Render<U>
where
    T: Component + Renderable<T>,
    <T as Component>::Properties: FromCaptures,
    U: for<'de> YewRouterState<'de>,
{
    Render::new(|captures: &Captures| {
        let props = T::Properties::from_captures(captures).ok()?;
        Some(create_component::<T, Router<U>>(props))
    })
}

/// Shorthand for [Render::new()](structs.Render.html#new).
pub fn render<T: for<'de> YewRouterState<'de>>(
    render: impl RenderFn<Router<T>> + 'static,
) -> Render<T> {
    Render::new(render)
}

/// A wrapper around a `RenderFn`.
/// This render function determines if a given route will succeed,
/// even after it has successfully matched a URL,
/// as well as controlling what will be rendered if it routes successfully.
#[derive(Clone)]
pub struct Render<T: for<'de> YewRouterState<'de>>(pub(crate) Option<Rc<dyn RenderFn<Router<T>>>>);

impl<T: for<'de> YewRouterState<'de>> Default for Render<T> {
    fn default() -> Self {
        Render(None)
    }
}

impl<T: for<'de> YewRouterState<'de>> Render<T> {
    /// Wraps a `RenderFn` in an optional Rc pointer, producing a new `Render`.
    pub fn new(render: impl RenderFn<Router<T>> + 'static) -> Self {
        Render(Some(Rc::new(render)))
    }
}

impl<T: for<'de> YewRouterState<'de>> Debug for Render<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("Render")
    }
}
