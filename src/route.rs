//! Wrapper around route url string, and associated history state.
#[cfg(feature = "service")]
use crate::service::RouteService;
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


#[cfg(feature = "service")]
/// Formats a path, query, and fragment into a string.
///
/// # Note
/// This expects that all three already have their expected separators (?, #, etc)
pub(crate) fn format_route_string(path: &str, query: &str, fragment: &str) -> String {
    format!(
        "{path}{query}{fragment}",
        path = path,
        query = query,
        fragment = fragment
    )
}

impl Route<()> {
    /// Creates a new route with no state out of a string.
    ///
    /// This Route will have `()` for its state.
    pub fn new_no_state<T: AsRef<str>>(route: T) -> Self {
        Route {
            route: route.as_ref().to_string(),
            state: None,
        }
    }
}

impl <T: Default> Route<T> {
    /// Creates a new route out of a string, setting the state to its default value.
    pub fn new_default_state<U: AsRef<str>>(route: U) -> Self {
        Route {
            route: route.as_ref().to_string(),
            state: Some(T::default()),
        }
    }
}

impl<T> fmt::Display for Route<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.route, f)
    }
}

// This is getting removed anyway
impl<T: Default> From<&str> for Route<T> {
    fn from(string: &str) -> Route<T> {
        Route {
            route: string.to_string(),
            state: T::default(),
        }
    }
}

impl<T> Deref for Route<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.route
    }
}
