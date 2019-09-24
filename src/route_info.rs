//! Wrapper around route url string, and associated history state.
use crate::route_service::RouteService;
use serde::Deserialize;
use serde::Serialize;
use stdweb::unstable::TryFrom;
use stdweb::JsSerialize;
use stdweb::Value;

//use std::ops::Deref;
use yew::agent::Transferable;

/// Any state that can be stored by the History API must meet the criteria of this trait.
pub trait RouteState: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}
impl<T> RouteState for T where T: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}


/// An abstraction over how the string for the router was created.
///
/// # Note
/// This is unstable and may be removed pre 1.0
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RouteString {
    /// An unstructured route string.
    Unstructured(String),
    /// A structured route string.
    Structured{
        /// The path.
        path: String,
        /// The query
        query: String,
        /// The Fragment
        fragment: String
    }
}

impl Default for RouteString {
    fn default() -> Self {
        RouteString::Unstructured(Default::default())
    }
}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RouteInfo<T> {
    /// The route string
    pub route: RouteString,
    /// The state
    pub state: Option<T>,
}

/// Formats a path, query, and fragment into a string.
///
/// # Note
/// This expects that all three already have their expected separators (?, #, etc)
pub (crate) fn format_route_string(path: &str, query: &str, fragment: &str) -> String {
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
        let path = route_service.get_path();
        let query = route_service.get_query();
        let fragment = route_service.get_fragment();
        RouteInfo {
            route: RouteString::Structured {
                path,
                query,
                fragment
            },
            state: None,
        }
    }

    /// Returns a string representation of the route.
    pub fn to_string(&self) -> String {
        match &self.route {
            RouteString::Unstructured(s) => s.to_owned(),
            RouteString::Structured {path, query, fragment} => format_route_string(path, query, fragment)
        }
    }

    /// Gets the path if the RouteInfo was constructed with a structured route string.
    ///
    /// # Note
    /// This expects that all three already have their expected separators (?, #, etc)
    pub fn get_path(&self) -> Option<&str> {
        if let RouteString::Structured {path, ..}  = &self.route {
            Some(&path)
        } else {
            None
        }
    }

    /// Gets the query if the RouteInfo was constructed with a structured route string.
    ///
    /// # Note
    /// This expects that all three already have their expected separators (?, #, etc)
    pub fn get_query(&self) -> Option<&str> {
        if let RouteString::Structured {query, ..}  = &self.route {
            Some(&query)
        } else {
            None
        }
    }
    /// Gets the fragment if the RouteInfo was constructed with a structured route string.
    ///
    /// # Note
    /// This expects that all three already have their expected separators (?, #, etc)
    pub fn get_fragment(&self) -> Option<&str> {
        if let RouteString::Structured {fragment, ..}  = &self.route {
            Some(&fragment)
        } else {
            None
        }
    }
}

impl<T> From<String> for RouteInfo<T> {
    fn from(string: String) -> RouteInfo<T> {
        RouteInfo {
            route: RouteString::Unstructured(string),
            state: None,
        }
    }
}

impl<T> From<&str> for RouteInfo<T> {
    fn from(string: &str) -> RouteInfo<T> {
        RouteInfo {
            route: RouteString::Unstructured(string.to_string()),
            state: None,
        }
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
