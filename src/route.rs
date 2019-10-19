//! Wrapper around route url string, and associated history state.
use crate::service::RouteService;
use serde::{Deserialize, Serialize};
use stdweb::{unstable::TryFrom, JsSerialize, Value};

// use std::ops::Deref;
use std::ops::Deref;
use yew::agent::Transferable;

/// Any state that can be stored by the History API must meet the criteria of this trait.
pub trait RouteState: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}
impl<T> RouteState for T where T: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Route<T> {
    /// The route string
    pub route: String,
    /// The state
    pub state: Option<T>,
}

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
        // stdweb.
        Route { route, state: None }
    }

    /// Returns a string representation of the route.
    pub fn to_string(&self) -> String {
        self.route.to_string()
    }
}

impl<T> From<String> for Route<T> {
    fn from(string: String) -> Route<T> {
        Route {
            route: string,
            state: None,
        }
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

impl<T> Transferable for Route<T> where for<'de> T: Serialize + Deserialize<'de> {}
