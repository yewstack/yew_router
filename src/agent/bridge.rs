//! Bridge to RouteAgent.
use crate::{agent::{RouteAgent}, route::Route, RouteState};
use std::{
    fmt::{Debug, Error as FmtError, Formatter},
    ops::{Deref, DerefMut},
};
use yew::{
    agent::{Bridged, Context},
    Bridge, Callback,
};

/// A wrapped bridge to the route agent.
///
/// A component that owns this can send and receive messages from the agent.
pub struct RouteAgentBridge<T = ()>(Box<dyn Bridge<RouteAgent<T>>>)
where
    T: RouteState;

impl<T> RouteAgentBridge<T>
where
    T: RouteState,
{
    /// Creates a new bridge.
    pub fn new(callback: Callback<Route<T>>) -> Self {
        let router_agent = RouteAgent::bridge(callback);
        RouteAgentBridge(router_agent)
    }

    /// Experimental, may be removed
    ///
    /// Directly spawn a new Router
    pub fn spawn(callback: Callback<Route<T>>) -> Self {
        use yew::agent::Discoverer;
        let router_agent = Context::spawn_or_join(Some(callback));
        RouteAgentBridge(router_agent)
    }
}

impl<T: RouteState> Debug for RouteAgentBridge<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteAgentBridge").finish()
    }
}

impl<T: RouteState> Deref for RouteAgentBridge<T> {
    type Target = Box<dyn Bridge<RouteAgent<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: RouteState> DerefMut for RouteAgentBridge<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
