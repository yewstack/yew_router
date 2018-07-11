use stdweb::Value;
use stdweb::JsSerialize;
use stdweb::unstable::TryFrom;
use serde::Serialize;
use serde::Deserialize;
use routing_service::RouteService;

use yew::agent::Transferable;

pub type Route = RouteBase<()>;

/// Any state that can be stored by the History API must meet the criteria of this trait.
pub trait RouteState: Clone + Default + JsSerialize + TryFrom<Value> + 'static {}
impl <T> RouteState for T
    where T: Clone + Default + JsSerialize + TryFrom<Value> + 'static
{}

/// The representation of a route, segmented into different sections for easy access.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize) ]
pub struct RouteBase<T> {
    pub path_segments: Vec<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
    pub state: T
}


impl<T> RouteBase<T>
    where T: RouteState
{
    /// Converts the Route to a string that is used to set the URL.
    pub fn to_route_string(&self) -> String {
        let path = self.path_segments.join("/");
        let mut path = format!("/{}", path); // add the leading '/'
        if let Some(ref query) = self.query {
            path = format!("{}?{}", path, query);
        }
        if let Some(ref fragment) = self.fragment {
            path = format!("{}#{}", path, fragment)
        }
        path
    }

    /// Gets the current route from the route service.
    pub fn current_route(route_service: &RouteService<T>) -> Self
    {
        let path = route_service.get_path(); // guaranteed to always start with a '/'
        let mut path_segments: Vec<String> = path.split("/").map(String::from).collect();
        path_segments.remove(0); // remove empty string that is split from the first '/'

        let mut query: String = route_service.get_query(); // The first character will be a '?'
        let query: Option<String> = if query.len() > 1 {
            query.remove(0);
            Some(query)
        } else {
            None
        };

        let mut fragment: String = route_service.get_fragment(); // The first character will be a '#'
        let fragment: Option<String> = if fragment.len() > 1 {
            fragment.remove(0);
            Some(fragment)
        } else {
            None
        };


        RouteBase {
            path_segments,
            query,
            fragment,
            state: T::default()
        }
    }

    /// Parse the string into a Route.
    pub fn parse<U: AsRef<str>>(string: U) -> RouteBase<T> {
        let string: &str = string.as_ref();
        let mut path_segments = vec![];
        let mut query = None;
        let mut fragment = None;
        let mut active_segment = String::new();

        #[derive(Clone, Copy)]
        enum RouteStateMachine {
            GettingPath,
            GettingQuery,
            GettingFragment
        }

        let mut state = RouteStateMachine::GettingPath;

        // sanitize string
        let string = string.trim_left_matches('/');

        // parse the route
        for character in string.chars() {
            match state {
                RouteStateMachine::GettingPath => {
                    match character {
                        '?' => state = {
                            path_segments.push(active_segment.clone());
                            active_segment = String::new();
                            RouteStateMachine::GettingQuery
                        },
                        '#' => state = {
                            path_segments.push(active_segment.clone());
                            active_segment = String::new();
                            RouteStateMachine::GettingFragment
                        },
                        '/' => {
                            path_segments.push(active_segment.clone());
                            active_segment = String::new()
                        },
                        any => active_segment.push(any)
                    }
                }
                RouteStateMachine::GettingQuery => {
                    match character {
                        '#' => state = {
                            query = Some(active_segment.clone());
                            active_segment = String::new();
                            RouteStateMachine::GettingFragment
                        },
                        any => active_segment.push(any)
                    }
                }
                RouteStateMachine::GettingFragment => active_segment.push(character)
            }
        }

        match state {
            RouteStateMachine::GettingPath => path_segments.push(active_segment.clone()),
            RouteStateMachine::GettingQuery =>    query = Some(active_segment.clone()),
            RouteStateMachine::GettingFragment => fragment = Some(active_segment.clone())
        }

        RouteBase {
            path_segments,
            query,
            fragment,
            state: T::default()
        }
    }
}

impl <T> From<String> for RouteBase<T>
    where T: RouteState
{
    fn from(string: String) -> RouteBase<T> {
        RouteBase::parse(string)
    }
}


impl <T> Transferable for RouteBase<T>
    where for <'de> T: Serialize + Deserialize<'de>
{}


/// A simple wrapper around format! that makes it easier to create `Route` structs.
#[macro_export]
macro_rules! route {
    ($($tts:tt)*) => {
        RouteBase::parse(format!($($tts)*))
    }
}

#[test]
fn route_macro() {
    let route = route!("/hello/world");
    assert_eq!(route, Route::parse("hello/world"));

    let world = "world";
    let route = route!("hello/{}", world);
    assert_eq!(route, Route::parse("hello/world"));
}