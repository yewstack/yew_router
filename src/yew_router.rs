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
    pub routes: Vec<ChildResolver>,
    pub routing_failed_page: Option<DefaultPage>
}

pub struct YewRouter {
    router: Box<Bridge<Router<()>>>,
    route: Option<Route<()>>,
    routes: Vec<ChildResolver>,
    routing_failed_page: Option<DefaultPage>
}

#[derive(Clone)]
pub struct ChildResolver(fn(route: &Route<()>) -> Option<VNode<YewRouter>>);

impl PartialEq for ChildResolver {
    fn eq(&self, other: &ChildResolver) -> bool {
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
        // I think the `Component` trait should have a hook called `on_mount()`
        // that is called after the component has been attached to the vdom.
        // It seems like this only works because the JS engine decides to activate the
        // router worker logic after the mounting has finished.
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
    fn tune_props_from_route(route: &Route<()>) -> Option<<Self as Component>::Properties>;

    fn encode_info_in_route(&self, _route: &mut Route<()>) {
    }
    /// This is a wrapped function pointer to a function that will create a component.
    const RESOLVER: ChildResolver = ChildResolver(resolve_child::<Self>);
}

/// For a component that allows its props to be constructed from the Route,
/// this function will instansiate the component within the context of the YewRouter.
fn resolve_child<T: Routable>(route: &Route<()>) -> Option<VNode<YewRouter>> {
    if let Some(props) = T::tune_props_from_route(route) {
        let mut comp = VComp::lazy::<T>().1;
        comp.set_props(props);
        return Some(VNode::VComp(comp))
    }

    return None
}

