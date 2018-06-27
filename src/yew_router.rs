//! Component that performs routing.

use yew::prelude::*;
use router::{Route, Router};
use yew::html::Component;
use router::Request as RouterRequest;

use yew::virtual_dom::VNode;
use yew::virtual_dom::VComp;
use yew::virtual_dom::VList;

pub enum Msg {
    HandleRoute(Route<()>)
}



#[derive(Clone, PartialEq, Default)]
pub struct Props {
    pub routes: Vec<ComponentConstructorAttempter>,
    pub routing_failed_page: Option<DefaultPage>
}

pub struct YewRouter {
    router: Box<Bridge<Router<()>>>,
    route: Option<Route<()>>,
    routes: Vec<ComponentConstructorAttempter>,
    routing_failed_page: Option<DefaultPage>
}

#[derive(Clone)]
pub struct ComponentConstructorAttempter(fn(route: &Route<()>) -> Option<VNode<YewRouter>>);

impl PartialEq for ComponentConstructorAttempter {
    fn eq(&self, other: &ComponentConstructorAttempter) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

#[derive(Clone)]
pub struct DefaultPage(pub fn(route: &Route<()>) -> VNode<YewRouter>);

impl PartialEq for DefaultPage {
    fn eq(&self, other: &DefaultPage) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}

impl Component for YewRouter {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|route: Route<()>| Msg::HandleRoute(route));
        let router = Router::bridge(callback);

        // TODO Not sure if this is technically correct. This should be sent _after_ the component has been created.
        router.send(RouterRequest::GetCurrentRoute);

        YewRouter {
            router,
            route: None,
            routes: props.routes,
            routing_failed_page: props.routing_failed_page
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::HandleRoute(route) => {
                self.route = Some(route);
                true
            }
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.routes = props.routes;
        self.routing_failed_page = props.routing_failed_page;
        true
    }
}


impl Renderable<YewRouter> for YewRouter {
    fn view(&self) -> Html<YewRouter> {

        if let Some(ref route) = self.route {
            for resolver in &self.routes {
                if let Some(child) = (resolver.0)(&route) {
                   return child
                }
            }
            if let Some(ref default_page) = self.routing_failed_page {
                (default_page.0)(&route) // router did not match - showing default
            } else {
                return VNode::VList(VList::new()) // empty - router did not match
            }
        } else {
            VNode::VList(VList::new()) // empty - no route yet
        }
    }
}

/// A trait that allows a component to be routed by a Yew router.
pub trait Routable: Component + Renderable<Self> {
    /// Try to construct the props used for creating a Component from route info.
    /// If None is returned, the router won't create the component.
    /// If Some(_) is returned, the router will create the component using the props
    /// and will stop trying to create other components.
    fn resolve_props(route: &Route<()>) -> Option<<Self as Component>::Properties>;

    /// This is a wrapped function pointer to a function that will try to create a component if the route matches.
    const ROUTING_CONSTRUCTOR_ATTEMPTER: ComponentConstructorAttempter = ComponentConstructorAttempter(try_construct_component_from_route::<Self>);
}

/// For a component that allows its props to be constructed from the Route,
/// this function will instansiate the component within the context of the YewRouter.
fn try_construct_component_from_route<T: Routable>(route: &Route<()>) -> Option<VNode<YewRouter>> {
    if let Some(props) = T::resolve_props(route) {
        let mut comp = VComp::lazy::<T>().1;
        comp.set_props(props);
        return Some(VNode::VComp(comp))
    }

    return None
}

/// Turns the provided component type name into a wrapped function that will create the component.
#[macro_export]
macro_rules! routes {
    ( $( $x:tt ),* ) => {
        vec![$(<($x)>::ROUTING_CONSTRUCTOR_ATTEMPTER )*]
    };
}
