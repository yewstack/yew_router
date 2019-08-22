//! Routing service
use crate::routing_service::RouteService;

use yew::prelude::worker::*;

use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

use crate::route::RouteInfo;
use crate::route::RouteState;
use yew::callback::Callback;
use log::trace;

/// Any state that can be used in the router agent must meet the criteria of this trait.
pub trait RouterState<'de>: RouteState + Serialize + Deserialize<'de> + Debug {}
impl<'de, T> RouterState<'de> for T where T: RouteState + Serialize + Deserialize<'de> + Debug {}

pub enum Msg<T>
where
    T: RouteState,
{
    BrowserNavigationRouteChanged((String, T)),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RouterRequest<T> {
    /// Replaces the most recent Route with a new one and alerts connected components to the route change.
    ReplaceRoute(RouteInfo<T>),
    /// Replaces the most recent Route with a new one, but does not alert connected components to the route change.
    ReplaceRouteNoBroadcast(RouteInfo<T>),
    /// Changes the route using a Route struct and alerts connected components to the route change.
    ChangeRoute(RouteInfo<T>),
    /// Changes the route using a Route struct, but does not alert connected components to the route change.
    ChangeRouteNoBroadcast(RouteInfo<T>),
    /// Gets the current route.
    GetCurrentRoute,
    /// Removes the entity from the Router Agent
    // TODO this is a temporary message because yew currently doesn't call the destructor, so it must be manually engaged
    Disconnect,
}

impl<T> Transferable for RouterRequest<T> where for<'de> T: Serialize + Deserialize<'de> {}

/// A simplified routerBase that assumes that no state is stored.
pub type Router = RouterBase<()>;
/// A simplified interface to the router agent
pub struct RouterBase<T>(Box<dyn Bridge<RouterAgent<T>>>)
where
    for<'de> T: RouterState<'de>;

pub type SimpleRouterAgent = RouterAgent<()>;

/// The Router agent holds on to the RouteService singleton and mediates access to it.
pub struct RouterAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    link: AgentLink<RouterAgent<T>>,
    route_service: RouteService<T>,
    /// A list of all entities connected to the router.
    /// When a route changes, either initiated by the browser or by the app,
    /// the route change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl<T> Agent for RouterAgent<T>
where
    for<'de> T: RouterState<'de>,
{
    type Reach = Context;
    type Message = Msg<T>;
    type Input = RouterRequest<T>;
    type Output = RouteInfo<T>;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.send_back(Msg::BrowserNavigationRouteChanged);
        let mut route_service = RouteService::new();
        route_service.register_callback(callback);

        RouterAgent {
            link,
            route_service,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::BrowserNavigationRouteChanged((_route_string, state)) => {
                trace!("Browser navigated");
                let mut route = RouteInfo::current_route(&self.route_service);
                route.state = Some(state);
                for sub in &self.subscribers {
                    self.link.response(*sub, route.clone());
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            RouterRequest::ReplaceRoute(route) => {
                let route_string: String = route.route;
                self.route_service.replace_route(&route_string, route.state.unwrap_or_default());
                let route = RouteInfo::current_route(&self.route_service);
                for sub in &self.subscribers {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ReplaceRouteNoBroadcast(route) => {
                let route_string: String = route.route;
                self.route_service.replace_route(&route_string, route.state.unwrap_or_default());
            }
            RouterRequest::ChangeRoute(route) => {
                let route_string: String = route.route;
                // set the route
                self.route_service.set_route(&route_string, route.state.unwrap_or_default());
                // get the new route. This will contain a default state object
                let route = RouteInfo::current_route(&self.route_service);
                // broadcast it to all listening components
                for sub in &self.subscribers {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ChangeRouteNoBroadcast(route) => {
                let route_string: String = route.route;
                self.route_service.set_route(&route_string, route.state.unwrap_or_default());
            }
            RouterRequest::GetCurrentRoute => {
                let route = RouteInfo::current_route(&self.route_service);
                self.link.response(who, route.clone());
            }
            RouterRequest::Disconnect => {
                self.disconnected(who);
            }
        }
    }
    fn disconnected(&mut self, id: HandlerId) {
        trace!(
            "request to disconnect; num subs: {}",
            self.subscribers.len()
        );
        self.subscribers.remove(&id);
        trace!(
            "disconnect processed ; num subs: {}",
            self.subscribers.len()
        ); // the latter value should be -1
           // if it doesn't then the handlerIds are different for each request
    }
}

impl<T> RouterBase<T>
where
    for<'de> T: RouterState<'de>,
{
    pub fn new(callback: Callback<RouteInfo<T>>) -> Self {
        let router_agent = RouterAgent::bridge(callback);
        RouterBase(router_agent)
    }

    /// Experimental, may be removed
    ///
    /// Directly spawn a new Router
    pub fn spawn(callback: Callback<RouteInfo<T>>) -> Self {
        use yew::agent::Discoverer;
        let router_agent = Context::spawn_or_join(callback);
        RouterBase(router_agent)
    }

    pub fn send(&mut self, request: RouterRequest<T>) {
        self.0.send(request)
    }
}

/// A sender for the Router that doesn't send messages back to the component that connects to it.
///
/// This may be subject to change
pub struct RouterSenderAgentBase<T>
where
    for<'de> T: RouterState<'de>,
{
    router_agent: Box<dyn Bridge<RouterAgent<T>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Void;
impl Transferable for Void {}

impl<T> Agent for RouterSenderAgentBase<T>
where
    for<'de> T: RouterState<'de>,
{
    type Reach = Context;
    type Message = ();
    type Input = RouterRequest<T>;
    type Output = Void;

    fn create(link: AgentLink<Self>) -> Self {
        RouterSenderAgentBase {
            router_agent: RouterAgent::bridge(link.send_back(|_| ())),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle(&mut self, msg: Self::Input, _who: HandlerId) {
        self.router_agent.send(msg);
    }
}

pub type RouterSender = RouterSenderBase<()>;

/// A simplified interface to the router agent
pub struct RouterSenderBase<T>(Box<dyn Bridge<RouterSenderAgentBase<T>>>)
where
    for<'de> T: RouterState<'de>;

impl<T> RouterSenderBase<T>
where
    for<'de> T: RouterState<'de>,
{
    pub fn new(callback: Callback<Void>) -> Self {
        let router_agent = RouterSenderAgentBase::bridge(callback);
        RouterSenderBase(router_agent)
    }

    pub fn send(&mut self, request: RouterRequest<T>) {
        self.0.send(request)
    }
}
