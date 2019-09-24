//! Service to handle routing.

use stdweb::web::event::PopStateEvent;
use stdweb::web::window;
use stdweb::web::EventListenerHandle;
use stdweb::web::History;
use stdweb::web::IEventTarget;
use stdweb::web::Location;
use stdweb::Value;
use yew::callback::Callback;

use crate::route_info::RouteState;
use std::marker::PhantomData;

/// A service that facilitates manipulation of the browser's URL bar and responding to browser
/// 'forward' and 'back' events.
///
/// The `T` determines what route state can be stored in the route service.
#[derive(Debug)]
pub struct RouteService<T> {
    history: History,
    location: Location,
    event_listener: Option<EventListenerHandle>,
    phantom_data: PhantomData<T>,
}

impl<T> Default for RouteService<T>
where
    T: RouteState,
{
    fn default() -> Self {
        RouteService::<T>::new()
    }
}

impl<T> RouteService<T> {
    /// Creates the route service.
    pub fn new() -> RouteService<T> {
        let location = window()
            .location()
            .expect("browser does not support location API");
        RouteService {
            history: window().history(),
            location,
            event_listener: None,
            phantom_data: PhantomData,
        }
    }

    #[inline]
    fn get_route_from_location(location: &Location) -> String {
        let path = location.pathname().unwrap();
        let query = location.search().unwrap();
        let fragment = location.hash().unwrap();
        crate::route_info::format_route_string(&path, &query, &fragment)
    }



    /// Gets the concatenated path, query, and fragment strings
    pub fn get_route(&self) -> String {
        Self::get_route_from_location(&self.location)
    }

    /// Gets the path name of the current url.
    pub fn get_path(&self) -> String {
        self.location.pathname().unwrap()
    }

    /// Gets the query string of the current url.
    pub fn get_query(&self) -> String {
        self.location.search().unwrap()
    }

    /// Gets the fragment of the current url.
    pub fn get_fragment(&self) -> String {
        self.location.hash().unwrap()
    }
}

impl<T> RouteService<T>
where
    T: RouteState,
{
    /// Registers a callback to the route service.
    /// Callbacks will be called when the History API experiences a change such as
    /// popping a state off of its stack when the forward or back buttons are pressed.
    pub fn register_callback(&mut self, callback: Callback<(String, T)>) {
        self.event_listener = Some(window().add_event_listener(move |event: PopStateEvent| {
            let state_value: Value = event.state();
            let state: T = T::try_from(state_value).unwrap_or_default();

            // Can't use the existing location, because this is a callback, and can't move it in here.
            let location: Location = window().location().unwrap();
            let route: String = Self::get_route_from_location(&location);

            callback.emit((route.clone(), state))
        }));
    }

    /// Sets the browser's url bar to contain the provided route,
    /// and creates a history entry that can be navigated via the forward and back buttons.
    /// The route should be a relative path that starts with a '/'.
    /// A state object be stored with the url.
    pub fn set_route(&mut self, route: &str, state: T) {
        self.history.push_state(state, "", Some(route));
    }

    /// Replaces the route with another one removing the most recent history event and
    /// creating another history event in its place.
    pub fn replace_route(&mut self, route: &str, state: T) {
        let _ = self.history.replace_state(state, "", Some(route));
    }
}
