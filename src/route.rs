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
    pub state: Option<T>,
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

#[cfg(feature = "service")]
impl<T> Route<T> {
    /// Gets the current route from the route service.
    ///
    /// # Note
    /// It does not get the current state.
    /// That is only provided via events.
    /// See [RouteService.register_callback](struct.RouteService.html#method.register_callback) to
    /// acquire state.
    pub fn current_route(route_service: &RouteService<T>) -> Self {
        let route = route_service.get_route();
        // TODO, should try to get the state using the history api once that is exposed through
        // stdweb. https://github.com/koute/stdweb/issues/371
        Route { route, state: None }
    }
}

impl<T> fmt::Display for Route<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.route, f)
    }
}

impl<T> From<&str> for Route<T> {
    fn from(string: &str) -> Route<T> {
        Route {
            route: string.to_string(),
            state: None,
        }
    }
}

impl<T> Deref for Route<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.route
    }
}
