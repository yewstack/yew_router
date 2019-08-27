use yew::{Component, Renderable, Html};
use yew::virtual_dom::{VNode, VComp};
use yew_router_path_matcher::{FromMatches, Matches, RenderFn};
use yew::virtual_dom::vcomp::ScopeHolder;
use crate::router::Router;
use crate::component_router::YewRouterState;

// TODO this is duplicated also in path matcher.
fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does, I may want a scope holder directly tied to the current context?
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}


/// Creates a render that creates the specified component if its
/// props can be created from the provided matches.
///
/// #Note
/// Allows specification of the router type.
pub fn component_s<T,U>() -> Render<U>
    where
        T: Component + Renderable<T>,
        <T as Component>::Properties: FromMatches,
        U: for<'de> YewRouterState<'de>
{
    Render::new(|matches: &Matches| {
        let props = T::Properties::from_matches(matches).ok()?;
        Some(create_component::<T, Router<U>>(props))
    })
}

/// Creates a render that creates the specified component if its
/// props can be created from the provided matches.
pub fn component<T>() -> Render<()>
    where
        T: Component + Renderable<T>,
        <T as Component>::Properties: FromMatches,
{
    component_s::<T, ()>()
}

/// Shorthand for [Render::new()](structs.Render.html#new).
pub fn render(render: impl RenderFn<Router<()>> + 'static) -> Render<()> {
    Render::new(render)
}


/// Shorthand for [Render::new()](structs.Render.html#new).
pub fn render_s<T: for<'de> YewRouterState<'de>>(render: impl RenderFn<Router<T>> + 'static) -> Render<T> {
    Render::new(render)
}




pub struct Render<T: for<'de> YewRouterState<'de>>(pub(crate) Option<Box<dyn RenderFn<Router<T>>>>);

impl <T: for<'de> YewRouterState<'de>> Default for Render<T> {
    fn default() -> Self {
        Render(None)
    }
}

impl <T: for<'de> YewRouterState<'de>> Render<T> {
    pub fn new(render: impl RenderFn<Router<T>> + 'static) -> Self {
        Render(Some(Box::new(render)))
    }
}
