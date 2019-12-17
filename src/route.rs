//! Wrapper around route url string, and associated history state.
#[cfg(feature = "service")]
use stdweb::{unstable::TryFrom, Value};
#[cfg(feature = "service")]
use serde::de::DeserializeOwned;

use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref};
use std::fmt::Debug;

/// Any state that can be used in the router agent must meet the criteria of this trait.
#[cfg(feature = "service")]
pub trait RouteState: Serialize + DeserializeOwned + Debug + Clone + Default + TryFrom<Value> + 'static {}
#[cfg(feature = "service")]
impl<T> RouteState for T where T: Serialize + DeserializeOwned + Debug + Clone + Default + TryFrom<Value> + 'static {}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Route<T = ()> {
    /// The route string
    pub route: String,
    /// The state stored in the history api
    pub state: T,
}


impl Route<()> {
    /// Creates a new route with no state out of a string.
    ///
    /// This Route will have `()` for its state.
    pub fn new_no_state<T: AsRef<str>>(route: T) -> Self {
        Route {
            route: route.as_ref().to_string(),
            state: (),
        }
    }
}

impl <T: Default> Route<T> {
    /// Creates a new route out of a string, setting the state to its default value.
    pub fn new_default_state<U: AsRef<str>>(route: U) -> Self {
        Route {
            route: route.as_ref().to_string(),
            state: T::default(),
        }
    }
}

impl<T> fmt::Display for Route<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.route, f)
    }
}

impl<T> Deref for Route<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.route
    }
}
