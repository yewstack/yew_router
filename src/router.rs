//! Router Component.

use crate::{
    agent::{RouteAgentBridge, RouteRequest},
    route::Route,
    Switch,
};
use std::{
    fmt::{self, Debug, Error as FmtError, Formatter},
    rc::Rc,
};
use yew::{html, virtual_dom::VNode, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::agent::AgentState;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait RouterState<'de>: AgentState<'de> + PartialEq {}
impl<'de, T> RouterState<'de> for T where T: AgentState<'de> + PartialEq {}

/// Rendering control flow component.
///
/// # Example
/// ```
/// use yew::{prelude::*, virtual_dom::VNode};
/// use yew_router::{router::Router, Switch};
///
/// pub enum Msg {}
///
/// pub struct Model {}
/// impl Component for Model {
///     //...
/// #   type Message = Msg;
/// #   type Properties = ();
/// #   fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
/// #       Model {}
/// #   }
/// #   fn update(&mut self, msg: Self::Message) -> ShouldRender {
/// #        false
/// #   }
///
///     fn view(&self) -> VNode {
///         html! {
///         <Router<(), S>
///            render = Router::render(|switch: S| {
///                match switch {
///                    S::Variant => html!{"variant route was matched"},
///                }
///            })
///         />
///         }
///     }
/// }
///
/// #[derive(Switch, Clone)]
/// enum S {
///     #[to = "/v"]
///     Variant,
/// }
/// ```
// TODO, can M just be removed due to not having to explicitly deal with callbacks anymore? - Just get rid of M
#[derive(Debug)]
pub struct Router<T: for<'de> RouterState<'de>, SW: Switch + Clone + 'static> {
    switch: Option<SW>,
    props: Props<T, SW>,
    router_agent: RouteAgentBridge<T>,
}

impl<T, SW> Router<T, SW>
where
    T: for<'de> RouterState<'de>,
    SW: Switch + Clone + 'static,
{
    // TODO render fn name is overloaded now with that of the trait: Renderable<_> this should be changed. Maybe: display, show, switch, inner...
    /// Wrap a render closure so that it can be used by the Router.
    /// # Example
    /// ```
    /// # use yew_router::Switch;
    /// # use yew_router::router::{Router, Render};
    /// # use yew::{html, Html};
    /// # #[derive(Switch, Clone)]
    /// # enum S {
    /// #     #[to = "/route"]
    /// #     Variant
    /// # }
    /// # pub enum Msg {}
    ///
    /// # fn dont_execute() {
    /// let render: Render<(), S> = Router::render(|switch: S| -> Html {
    ///     match switch {
    ///         S::Variant => html! {"Variant"},
    ///     }
    /// });
    /// # }
    /// ```
    pub fn render<F: RenderFn<Router<T, SW>, SW> + 'static>(f: F) -> Render<T, SW> {
        Render::new(f)
    }

    /// Wrap a redirect function so that it can be used by the Router.
    pub fn redirect<F: RedirectFn<SW, T> + 'static>(f: F) -> Option<Redirect<SW, T>> {
        Some(Redirect::new(f))
    }
}

/// Message for Router.
#[derive(Debug, Clone)]
pub enum Msg<T> {
    /// Updates the route
    UpdateRoute(Route<T>),
}

/// Render function that takes a switched route and converts it to HTML
pub trait RenderFn<CTX: Component, SW>: Fn(SW) -> Html {}
impl<T, CTX: Component, SW> RenderFn<CTX, SW> for T where T: Fn(SW) -> Html {}
/// Owned Render function.
#[derive(Clone)]
pub struct Render<T: for<'de> RouterState<'de>, SW: Switch + Clone + 'static>(
    pub(crate) Rc<dyn RenderFn<Router<T, SW>, SW>>,
);
impl<T: for<'de> RouterState<'de>, SW: Switch + Clone> Render<T, SW> {
    /// New render function
    fn new<F: RenderFn<Router<T, SW>, SW> + 'static>(f: F) -> Self {
        Render(Rc::new(f))
    }
}
impl<T: for<'de> RouterState<'de>, SW: Switch + Clone> Debug for Render<T, SW> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Render").finish()
    }
}

/// Redirection function that takes a route that didn't match any of the Switch variants,
/// and converts it to a switch variant.
pub trait RedirectFn<SW, STATE>: Fn(Route<STATE>) -> SW {}
impl<T, SW, STATE> RedirectFn<SW, STATE> for T where T: Fn(Route<STATE>) -> SW {}
/// Clonable Redirect function
#[derive(Clone)]
pub struct Redirect<SW: Switch + 'static, STATE: for<'de> RouterState<'de>>(
    pub(crate) Rc<dyn RedirectFn<SW, STATE>>,
);
impl<STATE: for<'de> RouterState<'de>, SW: Switch + 'static> Redirect<SW, STATE> {
    fn new<F: RedirectFn<SW, STATE> + 'static>(f: F) -> Self {
        Redirect(Rc::new(f))
    }
}
impl<STATE: for<'de> RouterState<'de>, SW: Switch> Debug for Redirect<SW, STATE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redirect").finish()
    }
}

/// Properties for Router.
#[derive(Properties, Clone)]
pub struct Props<T: for<'de> RouterState<'de>, SW: Switch + Clone + 'static> {
    /// Render function that takes a Switch and produces Html
    #[props(required)]
    pub render: Render<T, SW>,
    /// Optional redirect function that will convert the route to a known switch variant if explicit matching fails.
    /// This should mostly be used to handle 404s and redirection.
    /// It is not strictly necessary as your Switch is capable of handling unknown routes using `#[to="/{*:any}"]`.
    pub redirect: Option<Redirect<SW, T>>,
}

impl<T: for<'de> RouterState<'de>, SW: Switch + Clone> Debug for Props<T, SW> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("Props").finish()
    }
}

impl<T, SW> Component for Router<T, SW>
where
    T: for<'de> RouterState<'de>,
    SW: Switch + Clone + 'static,
{
    type Message = Msg<T>;
    type Properties = Props<T, SW>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::UpdateRoute);
        let router_agent = RouteAgentBridge::new(callback);

        Router {
            switch: Default::default(), /* This must be updated by immediately requesting a route
                                         * update from the service bridge. */
            props,
            router_agent,
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        self.router_agent.send(RouteRequest::GetCurrentRoute);
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateRoute(route) => {
                let mut switch = SW::switch(route.clone());

                if switch.is_none() {
                    if let Some(redirect) = &self.props.redirect {
                        let redirected: SW = (&redirect.0)(route);

                        log::trace!(
                            "Route failed to match, but redirecting route to a known switch."
                        );
                        // Replace the route in the browser with the redirected.
                        self.router_agent
                            .send(RouteRequest::ReplaceRouteNoBroadcast(
                                redirected.clone().into(),
                            ));
                        switch = Some(redirected)
                    }
                }

                self.switch = switch;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> VNode {
        match self.switch.clone() {
            Some(switch) => (&self.props.render.0)(switch),
            None => {
                log::warn!("No route matched, provide a redirect prop to the router to handle cases where no route can be matched");
                html! {"No route matched"}
            }
        }
    }
}
