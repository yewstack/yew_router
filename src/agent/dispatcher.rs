//! Dispatcher to RouteAgent.
use crate::agent::{RouteAgent};
use std::{
    fmt::{Debug, Error as FmtError, Formatter},
    ops::{Deref, DerefMut},
};
use yew::agent::{Dispatched, Dispatcher};
use crate::RouteState;

/// A wrapped dispatcher to the route agent.
///
/// A component that owns and instance of this can send messages to the RouteAgent, but not receive them.
pub struct RouteAgentDispatcher<T = ()>(Dispatcher<RouteAgent<T>>)
where
    T: RouteState;

impl<T> RouteAgentDispatcher<T>
where
    T: RouteState
{
    /// Creates a new bridge.
    pub fn new() -> Self {
        let dispatcher = RouteAgent::dispatcher();
        RouteAgentDispatcher(dispatcher)
    }
}

impl<T> Default for RouteAgentDispatcher<T>
where
    T: RouteState
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RouteState> Debug for RouteAgentDispatcher<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_tuple("RouteAgentDispatcher").finish()
    }
}

impl<T: RouteState> Deref for RouteAgentDispatcher<T> {
    type Target = Dispatcher<RouteAgent<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: RouteState> DerefMut for RouteAgentDispatcher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
