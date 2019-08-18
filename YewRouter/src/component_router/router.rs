use route::RouteInfo;
use router_agent::{RouterAgent, RouterRequest};
use yew::Bridged;
use yew::{
    html,
    virtual_dom::{vcomp::ScopeHolder, VComp, VNode},
    Bridge, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use YewRouterState;
use log::error;

pub trait FromRouteInfo<T> {
    fn from_route_info(path: &RouteInfo<T>) -> Option<Self>
    where
        Self: Sized;
}

fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(
    props: COMP::Properties,
) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does, I may want a scope holder directly tied to the current context?
    VNode::VComp(VComp::new::<COMP>(props, vcomp_scope))
}

pub struct Route<T, CONTEXT: Component> {
    /// Responsible for choosing if a route will be displayed and what will be displayed if it matches the RouteInfo.
    routing_and_display_fn: Box<dyn Fn(&RouteInfo<T>) -> Option<Html<CONTEXT>>>,
}

/// TODO, not sure if testing for pointer equality is the best option here
impl<T, CONTEXT: Component> PartialEq for Route<T, CONTEXT> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            self.routing_and_display_fn.as_ref(),
            other.routing_and_display_fn.as_ref(),
        )
    }
}

impl<T, CONTEXT: Component> Route<T, CONTEXT> {
    /// Takes a Fn that extracts the props for your chosen component from the route path.
    pub fn component<COMP, F>(routing_condition: F) -> Self
    where
        COMP: Component + Renderable<COMP>,
        F: Fn(&RouteInfo<T>) -> Option<<COMP as Component>::Properties> + 'static,
    {
        Route {
            routing_and_display_fn: Box::new(move |route: &RouteInfo<T>| {
                (routing_condition)(route).map(create_component::<COMP, CONTEXT>)
            }),
        }
    }

    /// If the properties implement FromPath<RouteBase<T>> then this can be used instead.
    pub fn component_from_route_info<'a, COMP>() -> Self
    where
        COMP: Component + Renderable<COMP>,
        COMP::Properties: FromRouteInfo<T>,
    {
        Route {
            routing_and_display_fn: Box::new(move |route: &RouteInfo<T>| {
                COMP::Properties::from_route_info(route).map(create_component::<COMP, CONTEXT>)
            }),
        }
    }

    /// If the routing condition returns Some(html) then the inner html will be rendered.
    pub fn render<F>(routing_condition: F) -> Self
    where
        F: Fn(&RouteInfo<T>) -> Option<Html<CONTEXT>> + 'static,
    {
        Route {
            routing_and_display_fn: Box::new(routing_condition),
        }
    }

    /// This option will be rendered regardless if the path matches the route.
    ///
    /// # Note
    /// This will prevent any route below from ever matching.
    pub fn children<F>(routing_condition: F) -> Self
    where
        F: Fn(&RouteInfo<T>) -> Html<CONTEXT> + 'static,
    {
        Route {
            routing_and_display_fn: Box::new(move |route: &RouteInfo<T>| {
                Some((routing_condition)(route))
            }),
        }
    }
}

/// Implementation of the router "algorithm".
/// Routes the first option to succeed or if all fail, will display nothing and log an error.
fn route_one_of<CONTEXT: Component, T: Clone>(
    route_options: &[Route<T, CONTEXT>],
    route: &RouteInfo<T>,
) -> Html<CONTEXT> {
    route_options
        .iter()
        .filter_map(|routing_option| (routing_option.routing_and_display_fn)(route))
        .next()
        .unwrap_or_else(|| {
            error!("Routing failed. No default case was provided.");
            html! { <></>}
        })
}

/// Router with state type of T
//pub struct Router<T: Default + PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + JsSerialize + StdwebTryFrom<Value> + Debug + 'static> {
pub struct Router<T: for<'de> YewRouterState<'de>> {
    route: RouteInfo<T>,
    route_options: Vec<Route<T, Router<T>>>,
    router_agent: Box<dyn Bridge<RouterAgent<T>>>,
}

pub enum Msg<T> {
    UpdateRoute(RouteInfo<T>),
}

#[derive(PartialEq, Properties)]
pub struct Props<T: for<'de> YewRouterState<'de>> {
    pub route_options: Vec<Route<T, Router<T>>>,
}

impl<T: for<'de> YewRouterState<'de>> Component for Router<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let router_agent = RouterAgent::bridge(callback);

        Router {
            route: Default::default(), // This must be updated by immediately requesting a route update from the service bridge.
            route_options: props.route_options,
            router_agent,
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        self.router_agent.send(RouterRequest::GetCurrentRoute);
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateRoute(route) => {
                let did_change = self.route != route;
                self.route = route;
                did_change
            }
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.route_options = props.route_options;
        true
    }
}

impl<T: for<'de> YewRouterState<'de> > Renderable<Router<T>> for Router<T>
{
    fn view(&self) -> VNode<Self> {
        route_one_of(&self.route_options, &self.route)
    }
}
