
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::virtual_dom::VComp;

use route::RouteBase;

use component_router::yew_router::YewRouterBase;
use component_router::YewRouterState;


/// A convenience trait that allows use of RoutableBase with the assumption that the state's type is ().
/// This should be used if you don't intend to be storing state in the History API.
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
///
/// If a component implements this trait, it can be utilized by the YewRouter with the following code:
/// '''
/// html! {
///     <YewRouterBase<T>: routes=routes![YourComponent, YourOtherComponent], />
/// }
/// '''
pub trait RoutableBase<T>: Component + Renderable<Self>
    where for<'de> T: YewRouterState<'de>
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


    /// This is set of function pointers to a function that will try to create a component if the route matches.
    const RESOLVER: ComponentResolverPackage<T> = ComponentResolverPackage {
        constructor_attempter: ComponentConstructorAttempter(try_construct_component_from_route::<T, Self>),
        will_try_to_route: ComponentWillTryToRoute(Self::will_try_to_route),
        will_route: ComponentWillRoute(will_route::<T, Self>)
    };
}

/// For a component that allows its props to be constructed from the Route,
/// this function will instansiate the component within the context of the YewRouter.
fn try_construct_component_from_route<T, R >(route: &RouteBase<T>) -> Option<VNode<YewRouterBase<T>>>
    where for<'de>
          T: YewRouterState<'de>,
          R: RoutableBase<T>
{
    R::resolve_props(route).map(|props| {
        let mut comp = VComp::lazy::<R>().1; // Creates a component
        comp.set_props(props); // The properties of the component _must_ be set
        VNode::VComp(comp)
    })
}

fn will_route<T, R>(route: &RouteBase<T>) -> bool
    where for<'de>
        T: YewRouterState<'de>,
        R: RoutableBase<T>
{
   R::resolve_props(route).is_some()
}

#[derive(Clone)]
pub struct ComponentConstructorAttempter<T>(pub(crate) fn(route: &RouteBase<T>) -> Option<Html<YewRouterBase<T>>>)
    where for<'de> T: YewRouterState<'de>
;

impl <T> PartialEq for ComponentConstructorAttempter<T>
    where for<'de> T: YewRouterState<'de>
{
    fn eq(&self, other: &ComponentConstructorAttempter<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

/// Components don't have to try to route.
/// If they try _and_ fail, then an error will be omitted, but otherwise a precondition must be met.
/// If the precondition is not met, then the component will not even be attempted to be routed, and no
/// error will be thrown.
#[derive(Clone)]
pub struct ComponentWillTryToRoute<T>(pub(crate) fn(route: &RouteBase<T>) -> bool);

impl <T> PartialEq for ComponentWillTryToRoute<T>
    where for<'de> T: YewRouterState<'de>
{
    fn eq(&self, other: &ComponentWillTryToRoute<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

/// This will tell if a component will route.
/// It is an optimization to prevent the outright construction of a component when it isn't used.
/// Instead, only the props are created and discarded, making this marginally cheaper
#[derive(Clone)]
pub struct ComponentWillRoute<T>(pub(crate) fn(route: &RouteBase<T>) -> bool);

impl <T> PartialEq for ComponentWillRoute<T>
    where for<'de> T: YewRouterState<'de>
{
    fn eq(&self, other: &ComponentWillRoute<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}



/// This is set of function pointers to a function that will try to create a component if the route matches.
#[derive(PartialEq, Clone)]
pub struct ComponentResolverPackage<T>
    where for<'de> T: YewRouterState<'de>
{
    pub(crate) constructor_attempter: ComponentConstructorAttempter<T>,
    pub(crate) will_try_to_route: ComponentWillTryToRoute<T>,
    pub(crate) will_route: ComponentWillRoute<T>
}