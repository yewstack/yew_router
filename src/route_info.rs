//! Wrapper around route url string, and associated history state.
use crate::route_service::RouteService;
use serde::Deserialize;
use serde::Serialize;
use stdweb::unstable::TryFrom;
use stdweb::JsSerialize;
use stdweb::Value;

//use std::ops::Deref;
use yew::agent::Transferable;
use std::ops::Deref;

/// Any state that can be stored by the History API must meet the criteria of this trait.
pub trait RouteState: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}
impl<T> RouteState for T where T: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RouteInfo<T> {
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

impl<T> RouteInfo<T> {
    /// Gets the current route from the route service.
    ///
    /// # Note
    /// It does not get the current state.
    /// That is only provided via events.
    /// See [RouteService.register_callback](struct.RouteService.html#method.register_callback) to acquire state.
    pub fn current_route(route_service: &RouteService<T>) -> Self {
        let route = route_service.get_route();
        // TODO, should try to get the state using the history api once that is exposed through stdweb.
        RouteInfo {
            route,
            state: None,
        }
    }

    /// Returns a string representation of the route.
    pub fn to_string(&self) -> String {
        self.route.to_string()
    }
}

impl<T> From<String> for RouteInfo<T> {
    fn from(string: String) -> RouteInfo<T> {
        RouteInfo {
            route: string,
            state: None,
        }
    }
}

impl<T> From<&str> for RouteInfo<T> {
    fn from(string: &str) -> RouteInfo<T> {
        RouteInfo {
            route: string.to_string(),
            state: None,
        }
    }
}

impl <T> Deref for RouteInfo<T> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.route
    }
}


//impl<T> Deref for RouteInfo<T> {
//    type Target = str;
//
//    fn deref(&self) -> &Self::Target {
//        &self.route
//    }
//}

impl<T> Transferable for RouteInfo<T> where for<'de> T: Serialize + Deserialize<'de> {}
