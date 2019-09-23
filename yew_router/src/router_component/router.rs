//! Router Component.

use crate::router_component::route::Route;
use crate::agent::{bridge::RouteAgentBridge, RouteRequest};
use crate::route_info::RouteInfo;
use crate::YewRouterState;
use log::{trace, warn};
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::rc::Rc;
use yew::html::ChildrenWithProps;
use yew::virtual_dom::VChild;
use yew::{
    html, virtual_dom::VNode, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use crate::matcher::RenderFn;

/// Rendering control flow component.
///
/// Based on the current url and its child [Routes](struct.Route.html), it will choose one route and
/// render its associated component.
///
///
/// # Example
/// ```
/// use yew::prelude::*;
/// use yew_router::{Router, Route, route, component};
/// use yew_router::FromCaptures;
///
/// pub struct AComponent {}
///
/// #[derive(Properties, FromCaptures)]
/// pub struct AComponentProps {
///     value: String,
///     other: Option<String>
/// }
///
/// impl Component for AComponent {
/// # type Message = ();
///    type Properties = AComponentProps;
///    //...
/// # fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {
/// #        unimplemented!()
/// #    }
/// # fn update(&mut self,msg: Self::Message) -> bool {
/// #        unimplemented!()
/// #    }
/// }
/// # impl Renderable<AComponent> for AComponent {
///  #     fn view(&self) -> Html<Self> {
/// #        unimplemented!()
/// #    }
///# }
///
/// pub struct Model {}
/// impl Component for Model {
///     //...
/// #   type Message = ();
/// #   type Properties = ();
/// #   fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
/// #       Model {}
/// #   }
/// #   fn update(&mut self, msg: Self::Message) -> ShouldRender {
/// #        false
/// #   }
/// }
///
/// impl Renderable<Model> for Model {
///     fn view(&self) -> Html<Self> {
///         html! {
///             <Router>
///                 <Route matcher=route!("/a/{value}") render=component::<AComponent>() />
///             </Router>
///         }
///     }
/// }
/// ```
pub struct Router<T: for<'de> YewRouterState<'de>> {
    route: RouteInfo<T>,
    props: Props<T>,
    router_agent: RouteAgentBridge<T>,
}

impl<T: for<'de> YewRouterState<'de>> Debug for Router<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("Router")
            .field("route", &self.props)
            .field("props", &self.props)
            .field("router_agent", &"Bridge to RouteAgent")
            .finish()
    }
}

/// Message for Router.
#[derive(Debug, Clone)]
pub enum Msg<T> {
    /// Updates the route
    UpdateRoute(RouteInfo<T>),
}

/// Properties for Router.
#[derive(Properties)]
pub struct Props<T: for<'de> YewRouterState<'de>> {
    #[props(required)]
    children: ChildrenWithProps<Route<T>, Router<T>>,
}

impl<T: for<'de> YewRouterState<'de>> Debug for Props<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.debug_struct("Props")
            .field("children (length)", &self.children.len())
            .finish()
    }
}

impl<T> Component for Router<T>
where
    T: for<'de> YewRouterState<'de>,
{
    type Message = Msg<T>;
    type Properties = Props<T>;

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
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl<T: for<'de> YewRouterState<'de>> Renderable<Router<T>> for Router<T> {
    fn view(&self) -> VNode<Self> {
        trace!(
            "Routing one of {} routes for  {:?}",
            self.props.children.iter().count(),
            &self.route
        );

        self.props
            .children
            .iter()
            .filter_map(|route| -> Option<Html<Self>> { try_render_child(route, &self.route) })
            .next() // Take the first path that succeeds.
            .map(|x| -> Html<Self> {
                trace!("Route matched.");
                x
            })
            .unwrap_or_else(|| {
                warn!("Routing failed. No default case was provided.");
                html! { <></>}
            })
    }
}

/// Tries to render a child.
///
/// It will run the route string against the matcher provided by the `route_child`.
/// If it matches, it will attempt to render content using either the render fn or the children..
///
/// # Arguments
/// * route_child - The child attempting to be rendered.
/// * route_string - The string representing the route.
fn try_render_child<T: for<'de> YewRouterState<'de>>(
    route_child: VChild<Route<T>, Router<T>>,
    route_string: &str,
) -> Option<Html<Router<T>>> {
    let children_present: bool = !route_child.props.children.is_empty();

    let children = route_child.props.children.iter();
    let render: Option<Rc<dyn RenderFn<Router<T>>>> = route_child.props.render.clone().0;

    route_child
        .props
        .matcher
        .match_route_string(route_string)
        .map(
            |matches:  std::collections::HashMap<&str, String>| {
                match render {
                    Some(render) => {
                        if children_present {
                            match (render)(&matches) {
                                Some(rendered) => Some(html! {
                                    <>
                                        {rendered}
                                        {children.collect::<VNode<Router<T>>>()}
                                    </>
                                }),
                                None => {
                                    // If the component can't be created from the matches,
                                    // the nested children will be rendered anyways
                                    Some(children.collect())
                                }
                            }
                        } else {
                            render(&matches)
                        }
                    }
                    None => {
                        if children_present {
                            Some(children.collect())
                        } else {
                            None // Neither matched
                        }
                    }
                }
            },
        )
        .flatten_stable()
}

trait Flatten<T> {
    /// Because flatten is a nightly feature. I'm making a new variant of the function here for stable use.
    /// The naming is changed to avoid this getting clobbered when object_flattening 60258 is stabilized.
    fn flatten_stable(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten_stable(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}
