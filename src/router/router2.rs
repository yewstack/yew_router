//! Router Component.

use crate::agent::{bridge::RouteAgentBridge, RouteRequest};
use crate::route_info::RouteInfo;
use crate::router::RouterState;
use crate::Switch;
use std::fmt::{self, Debug, Error as FmtError, Formatter};
use std::rc::Rc;
use yew::{
    virtual_dom::VNode, Callback, Component, ComponentLink, Html, Properties, Renderable,
    ShouldRender,
};

/// Rendering control flow component.
///
/// Based on the current url and its child [Routes](struct.Route.html), it will choose one route and
/// render its associated component.
///
///
/// # Example
/// ```
/// use yew::prelude::*;
/// use yew_router::router::router2::Router;
/// use yew_router::Switch;
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
///     Variant
/// }
///
/// impl Renderable<Model> for Model {
///     fn view(&self) -> Html<Self> {
///         html! {
///             <Router<(), S, Msg>
///                callback = From::from
///                render = Router::render(|switch: Option<&S>| {
///                    match switch {
///                        Some(S::Variant) => html!{"variant route was matched"},
///                        _ => unimplemented!()
///                    }
///                })
///             />
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Router<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> {
    route: RouteInfo<T>,
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
    ///# use yew_router::Switch;
    ///# use yew_router::router::router2::Router;
    ///# use yew::{html, Html};
    ///# #[derive(Switch)]
    ///# enum S {
    ///#     #[to = "/route"]
    ///#     Variant
    ///# }
    ///
    ///# fn dont_execute() {
    /// let render = Router::render(|switch: Option<&S>| -> Html<Router<(), S, ()>> {
    ///    match switch {
    ///        Some(S::Variant) => html!{"Variant"},
    ///        None => html!{"404"}
    ///    }
    /// });
    ///# }
    /// ```
    pub fn render<F: RenderFn2<Router<T, SW, M>, SW> + 'static>(f: F) -> Render2<T, SW, M> {
        Render2::new(f)
    }
}

/// Message for Router.
#[derive(Debug, Clone)]
pub enum Msg<T, M> {
    /// Updates the route
    UpdateRoute(RouteInfo<T>),
    /// Inner message
    InnerMessage(M),
}

impl<T, M> From<M> for Msg<T, M> {
    fn from(inner: M) -> Self {
        Msg::InnerMessage(inner)
    }
}

// TODO consider removing the Option, and creating two different render functions - one for rendering the switch, and one for a 404 case.
/// Render function definition
pub trait RenderFn2<CTX: Component, SW>: Fn(Option<&SW>) -> Html<CTX> {}
impl<T, CTX: Component, SW> RenderFn2<CTX, SW> for T where T: Fn(Option<&SW>) -> Html<CTX> {}
/// Owned Render function.
pub struct Render2<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static>(
    pub(crate) Rc<dyn RenderFn2<Router<T, SW, M>, SW>>,
);
impl<T: for<'de> RouterState<'de>, SW: Switch, M> Render2<T, SW, M> {
    /// New render function
    fn new<F: RenderFn2<Router<T, SW, M>, SW> + 'static>(f: F) -> Self {
        Render2(Rc::new(f))
    }
}
impl<T: for<'de> RouterState<'de>, SW: Switch, M> Debug for Render2<T, SW, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Render2").finish()
    }
}

/// Properties for Router.
#[derive(Properties)]
pub struct Props<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> {
    /// Render fn
    #[props(required)]
    pub render: Render2<T, SW, M>,
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
            route: Default::default(), // This must be updated by immediately requesting a route update from the service bridge.
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
        true // TODO, this can probably be better now.
    }
}

impl<T: for<'de> RouterState<'de>, SW: Switch + 'static, M: 'static> Renderable<Router<T, SW, M>>
    for Router<T, SW, M>
{
    fn view(&self) -> VNode<Self> {
        let switch = SW::switch(self.route.clone());
        (&self.props.render.0)(switch.as_ref())
    }
}
