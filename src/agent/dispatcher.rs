//! Bridge to RouteAgent.
use crate::agent::{AgentState, RouteAgent};
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::ops::{Deref, DerefMut};
use yew::agent::{Dispatched, Dispatcher};

/// A simplified interface to the router agent.
pub struct RouteAgentDispatcher<T>(Dispatcher<RouteAgent<T>>)
where
    for<'de> T: AgentState<'de>;

impl<T> RouteAgentDispatcher<T>
where
    for<'de> T: AgentState<'de>,
{
    /// Creates a new bridge.
    pub fn new() -> Self {
        let dispatcher = RouteAgent::dispatcher();
        RouteAgentDispatcher(dispatcher)
    }
}

/// A wrapper around the bridge
//pub (crate) struct RouteAgentBridge<T: for<'de> YewRouterState<'de>>(pub Box<dyn Bridge<RouteAgent<T>>>);

impl<T: for<'de> AgentState<'de>> Debug for RouteAgentDispatcher<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteAgentDispatcher").finish()
    }
}

impl<T: for<'de> AgentState<'de>> Deref for RouteAgentDispatcher<T> {
    type Target = Dispatcher<RouteAgent<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: for<'de> AgentState<'de>> DerefMut for RouteAgentDispatcher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
