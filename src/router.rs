//! Routing service
use routing_service::RouteService;

use yew::prelude::worker::*;

use std::collections::HashSet;

use stdweb::Value;
use stdweb::JsSerialize;
use stdweb::unstable::TryFrom;
use serde::Serialize;
use serde::Deserialize;
use std::fmt::Debug;

use route::RouteBase;

pub enum Msg<T>
    where T: JsSerialize + Clone + Debug + TryFrom<Value> + 'static
{
    BrowserNavigationRouteChanged((String, T)),
}




#[derive(Serialize, Deserialize, Debug)]
pub enum RouterRequest<T> {
    /// Replaces the most recent Route with a new one and alerts connected components to the route change.
    ReplaceRoute(RouteBase<T>),
    /// Replaces the most recent Route with a new one, but does not alert connected components to the route change.
    ReplaceRouteNoBroadcast(RouteBase<T>),
    /// Changes the route using a Route struct and alerts connected components to the route change.
    ChangeRoute(RouteBase<T>),
    /// Changes the route using a Route struct, but does not alert connected components to the route change.
    ChangeRouteNoBroadcast(RouteBase<T>),
    /// Gets the current route.
    GetCurrentRoute
}

impl <T> Transferable for RouterRequest<T>
    where for <'de> T: Serialize + Deserialize<'de>
{}

/// The Router agent holds on to the RouteService singleton and mediates access to it.
pub struct Router<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static
{
    link: AgentLink<Router<T>>,
    route_service: RouteService<T>,
    /// A list of all entities connected to the router.
    /// When a route changes, either initiated by the browser or by the app,
    /// the route change will be broadcast to all listening entities.
    subscribers: HashSet<HandlerId>,
}

impl<T> Agent for Router<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static
{
    type Reach = Context;
    type Message = Msg<T>;
    type Input = RouterRequest<T>;
    type Output = RouteBase<T>;

    fn create(link: AgentLink<Self>) -> Self {
        let callback = link.send_back(|route_changed: (String, T)| Msg::BrowserNavigationRouteChanged(route_changed));
        let mut route_service = RouteService::new();
        route_service.register_callback(callback);

        Router {
            link,
            route_service,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::BrowserNavigationRouteChanged((_route_string, state)) => {
                info!("Browser navigated");
                let mut route = RouteBase::current_route(&self.route_service);
                route.state = state;
                for sub in self.subscribers.iter() {
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
                let route_string: String = route.to_route_string();
                self.route_service.replace_route(&route_string, route.state);
                let route = RouteBase::current_route(&self.route_service);
                for sub in self.subscribers.iter() {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ReplaceRouteNoBroadcast(route) => {
                let route_string: String = route.to_route_string();
                self.route_service.replace_route(&route_string, route.state);
            }
            RouterRequest::ChangeRoute(route) => {
                let route_string: String = route.to_route_string();
                // set the route
                self.route_service.set_route(&route_string, route.state);
                // get the new route. This will contain a default state object
                let route = RouteBase::current_route(&self.route_service);
                // broadcast it to all listening components
                for sub in self.subscribers.iter() {
                    self.link.response(*sub, route.clone());
                }
            }
            RouterRequest::ChangeRouteNoBroadcast(route) => {
                let route_string: String = route.to_route_string();
                self.route_service.set_route(&route_string, route.state);
            }
            RouterRequest::GetCurrentRoute => {
                let route = RouteBase::current_route(&self.route_service);
                self.link.response(who, route.clone());
            }
        }
    }
    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
