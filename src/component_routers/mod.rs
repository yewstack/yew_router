mod yew_router;

use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::virtual_dom::VComp;

use router::RouteBase;

pub use self::yew_router::{YewRouterBase, YewRouter, DefaultPage, PropsBase, Props};
use stdweb::JsSerialize;
use std::fmt::Debug;
use stdweb::unstable::TryFrom;
use stdweb::Value;
use serde::Serialize;
use serde::Deserialize;

pub trait Routable: Component + Renderable<Self> {
    fn resolve_props(route: &RouteBase<()>) -> Option<<Self as Component>::Properties>;
    fn will_try_to_route(route: &RouteBase<()>) -> bool;
}

impl <T> RoutableBase<()> for T where T: Routable {
    #[inline]
    fn resolve_props(route: &RouteBase<()>) -> Option<<Self as Component>::Properties> {
        T::resolve_props(route)
    }
    #[inline]
    fn will_try_to_route(route: &RouteBase<()>) -> bool {
        T::will_try_to_route(route)
    }
}

/// A trait that allows a component to be routed by a Yew router.
pub trait RoutableBase<T>: Component + Renderable<Self>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    /// Try to construct the props used for creating a Component from route info.
    ///
    /// If Some, then the YewRouter will construct the component.
    /// If None, then the YewRouter will emit an error to any other listening YewRouter, telling it to
    /// display the error page if it is configured to do so.
    ///
    /// If Empty is returned, the router won't create the component.
    /// If NoMatch is returned, the router won't create the component, and will also propagate a
    /// message to any listening `YewRouter` to tell itself that routing has failed.
    /// If Matched(_) is returned, the router will create the component using the props
    /// and will stop trying to create other components.
    ///
    /// Empty should be returned if the get(index) on the path segments fails.
    /// If NoMatch is returned in this place, the whole routing suite will break.
    /// This sensitivity to programmer error make it a good idea to derive this trait instead of
    /// implementing it yourself.
    fn resolve_props(route: &RouteBase<T>) -> Option<<Self as Component>::Properties>;


    /// This if this returns false, `resolve_props` will not run, so unwrapping can be performed
    /// in resolve_props without fear of panicing.
    fn will_try_to_route(route: &RouteBase<T>) -> bool;


    /// This is a wrapped function pointer to a function that will try to create a component if the route matches.
    const RESOLVER: ComponentResolverPackage<T> = ComponentResolverPackage {
        constructor_attempter: ComponentConstructorAttempter(try_construct_component_from_route::<T, Self>),
        will_try_to_route: ComponentWillTryToRoute(Self::will_try_to_route),
    };
}

/// For a component that allows its props to be constructed from the Route,
/// this function will instansiate the component within the context of the YewRouter.
fn try_construct_component_from_route<T, R: RoutableBase<T>>(route: &RouteBase<T>) -> Option<VNode<YewRouterBase<T>>>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    R::resolve_props(route).map(|props| {
        let mut comp = VComp::lazy::<R>().1; // Creates a component
        comp.set_props(props); // The properties of the component _must_ be set
        VNode::VComp(comp)
    })
}

#[derive(Clone)]
pub struct ComponentConstructorAttempter<T>(fn(route: &RouteBase<T>) -> Option<VNode<YewRouterBase<T>>>)
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
;

impl <T> PartialEq for ComponentConstructorAttempter<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    fn eq(&self, other: &ComponentConstructorAttempter<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

#[derive(Clone)]
pub struct ComponentWillTryToRoute<T>(fn(route: &RouteBase<T>) -> bool);

impl <T> PartialEq for ComponentWillTryToRoute<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    fn eq(&self, other: &ComponentWillTryToRoute<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

#[derive(PartialEq, Clone)]
pub struct ComponentResolverPackage<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    constructor_attempter: ComponentConstructorAttempter<T>,
    will_try_to_route: ComponentWillTryToRoute<T>
}