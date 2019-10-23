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
use yew::{
    virtual_dom::VNode, Callback, Component, ComponentLink, Html, Properties, Renderable,
    ShouldRender, html
};
use std::marker::PhantomData;


use crate::agent::AgentState;

/// Any state that can be managed by the `Router` must meet the criteria of this trait.
pub trait RouterState<'de>: AgentState<'de> + PartialEq {}
impl<'de, T> RouterState<'de> for T where T: AgentState<'de> + PartialEq {}

/// Rendering control flow component.
///
/// # Example
/// ```
/// use yew::prelude::*;
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
/// }
///
/// #[derive(Switch)]
/// enum S {
///     #[to = "/v"]
///     Variant,
/// }
///
/// impl Renderable<Model> for Model {
///     fn view(&self) -> Html<Self> {
///         html! {
///             <Router<(), S, Msg>
///                callback = From::from
///                render = Router::render(|switch: S| {
///                    match switch {
///                        S::Variant => html!{"variant route was matched"},
///                    }
///                })
///             />
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Router<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> {
    route: Route<T>,
    props: Props<T, SW, M>,
    router_agent: RouteAgentBridge<T>,
}

impl<T, SW, M> Router<T, SW, M>
where
    T: for<'de> RouterState<'de>,
    SW: Switch + 'static,
    M: 'static,
{
    /// Wrap a render closure so that it can be used by the Router.
    /// # Example
    /// ```
    /// # use yew_router::Switch;
    /// # use yew_router::router::Router;
    /// # use yew::{html, Html};
    /// # #[derive(Switch)]
    /// # enum S {
    /// #     #[to = "/route"]
    /// #     Variant
    /// # }
    /// # pub enum Msg {}
    ///
    /// # fn dont_execute() {
    /// let render = Router::render(|switch: S| -> Html<Router<(), S, Msg>> {
    ///     match switch {
    ///         S::Variant => html! {"Variant"},
    ///     }
    /// });
    /// # }
    /// ```
    pub fn render<F: RenderFn<Router<T, SW, M>, SW> + 'static>(f: F) -> Render<T, SW, M> {
        Render::new(f)
    }


    /// Wrap a redirect function so that it can be used by the Router.
    pub fn redirect<F: RedirectFn<SW, T> + 'static>(f: F) -> Option<Redirect<SW, T, M>> {
        Some(Redirect::new(f))
    }
}


/// Message for Router.
#[derive(Debug, Clone)]
pub enum Msg<T, M> {
    /// Updates the route
    UpdateRoute(Route<T>),
    /// Inner message
    InnerMessage(M),
}

impl<T, M> From<M> for Msg<T, M> {
    fn from(inner: M) -> Self {
        Msg::InnerMessage(inner)
    }
}

/// Render function that takes a switched route and converts it to HTML
pub trait RenderFn<CTX: Component, SW>: Fn(SW) -> Html<CTX> {}
impl<T, CTX: Component, SW> RenderFn<CTX, SW> for T where T: Fn(SW) -> Html<CTX> {}
/// Owned Render function.
pub struct Render<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static>(
    pub(crate) Rc<dyn RenderFn<Router<T, SW, M>, SW>>,
);
impl<T: for<'de> RouterState<'de>, SW: Switch, M> Render<T, SW, M> {
    /// New render function
    fn new<F: RenderFn<Router<T, SW, M>, SW> + 'static>(f: F) -> Self {
        Render(Rc::new(f))
    }
}
impl<T: for<'de> RouterState<'de>, SW: Switch, M> Debug for Render<T, SW, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Render").finish()
    }
}

/// Redirection function that takes a route that didn't match any of the Switch variants,
/// and converts it to a switch variant.
pub trait RedirectFn<SW, STATE>: Fn(Route<STATE>)-> SW {}
impl<T, SW, STATE,> RedirectFn<SW, STATE> for T where T: Fn(Route<STATE>)-> SW {}
/// Clonable Redirect function
pub struct Redirect<SW: Switch + 'static, STATE: for<'de> RouterState<'de>, M>(
    pub(crate) Rc<dyn RedirectFn<SW, STATE>>,
    /// This phantom data is here to allow type inference when using it inside a Router component.
    PhantomData<M>
);
impl<STATE: for<'de> RouterState<'de>, SW: Switch + 'static, M> Redirect<SW, STATE, M> {
    fn new<F: RedirectFn<SW, STATE> + 'static>(f: F) -> Self {
        Redirect(Rc::new(f), PhantomData)
    }
}
impl<STATE: for<'de> RouterState<'de>, SW: Switch, M> Debug for Redirect<SW, STATE, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redirect").finish()
    }
}

/// Properties for Router.
#[derive(Properties)]
pub struct Props<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> {
    /// Render function that
    #[props(required)]
    pub render: Render<T, SW, M>,
    /// Optional redirect function that will convert the route to a known switch variant if explicit matching fails.
    /// This should mostly be used to handle 404s and redirection.
    /// It is not strictly necessary as your Switch is capable of handling unknown routes using `#[to="/{*:any}"]`.
    pub redirect: Option<Redirect<SW, T, M>>,
    /// Optional Callback for propagating messages to parent components.
    pub callback: Option<Callback<M>>,
}

impl<T: for<'de> RouterState<'de>, SW: Switch, M> Debug for Props<T, SW, M> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("Props").finish()
    }
}

impl<T, SW, M> Component for Router<T, SW, M>
where
    T: for<'de> RouterState<'de>,
    SW: Switch + 'static,
    M: 'static,
{
    type Message = Msg<T, M>;
    type Properties = Props<T, SW, M>;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let router_agent = RouteAgentBridge::new(callback);

        Router {
            route: Default::default(), /* This must be updated by immediately requesting a route
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
                let did_change = self.route != route;
                self.route = route;
                did_change
            }
            Msg::InnerMessage(m) => {
                if let Some(cb) = &self.props.callback {
                    cb.emit(m)
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> Renderable<Router<T, SW, M>>
    for Router<T, SW, M>
{
    fn view(&self) -> VNode<Self> {
        let switch: Option<SW> = SW::switch(self.route.clone());
        match switch {
            Some(switch) => (&self.props.render.0)(switch),
            None => {
                if let Some(redirect_fn) = &self.props.redirect {
                    let switch: SW = (redirect_fn.0)(self.route.clone()); // TODO This should be used to set the route
                    (&self.props.render.0)(switch)
                } else {
                    html!{format!{"No route for {}", self.route.route}}
                }
            }
        }
    }
}
