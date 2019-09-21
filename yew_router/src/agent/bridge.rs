//! Bridge to RouteAgent.
use yew::{Bridge, Callback};
use yew::agent::Context;
use std::fmt::{Debug, Formatter, Error as FmtError};
use std::ops::{Deref, DerefMut};
use crate::route_info::RouteInfo;
use crate::agent::{RouterState, RouteAgent};
use yew::agent::Bridged;




/// A simplified interface to the router agent.
pub struct RouteAgentBridge<T>(Box<dyn Bridge<RouteAgent<T>>>)
    where
            for<'de> T: RouterState<'de>;

impl<T> RouteAgentBridge<T>
    where
            for<'de> T: RouterState<'de>,
{
    /// Creates a new bridge.
    pub fn new(callback: Callback<RouteInfo<T>>) -> Self {
        let router_agent = RouteAgent::bridge(callback);
        RouteAgentBridge(router_agent)
    }

    /// Experimental, may be removed
    ///
    /// Directly spawn a new Router
    pub fn spawn(callback: Callback<RouteInfo<T>>) -> Self {
        use yew::agent::Discoverer;
        let router_agent = Context::spawn_or_join(callback);
        RouteAgentBridge(router_agent)
    }
}

/// A wrapper around the bridge
//pub (crate) struct RouteAgentBridge<T: for<'de> YewRouterState<'de>>(pub Box<dyn Bridge<RouteAgent<T>>>);

impl<T: for<'de> RouterState<'de>> Debug for RouteAgentBridge<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteAgentBridge").finish()
    }
}

impl<T: for<'de> RouterState<'de>> Deref for RouteAgentBridge<T> {
    type Target = Box<dyn Bridge<RouteAgent<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: for<'de> RouterState<'de>> DerefMut for RouteAgentBridge<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}