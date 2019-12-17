//! Wrapper around route url string, and associated history state.
use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref};
use stdweb::{unstable::TryFrom, Value};
use std::fmt::Debug;
use serde::de::DeserializeOwned;

/// Any state that can be used in the router agent must meet the criteria of this trait.
pub trait RouteState: Serialize + DeserializeOwned + Debug + Clone + Default + TryFrom<Value> + 'static {}
impl<T> RouteState for T where T: Serialize + DeserializeOwned + Debug + Clone + Default + TryFrom<Value> + 'static {}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Route<T = ()> {
    /// The route string
    pub route: String,
    /// The state stored in the history api
    pub state: T,
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
