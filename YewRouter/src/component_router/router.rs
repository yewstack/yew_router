//! Router Component.

use crate::route_info::RouteInfo;
use crate::route_agent::{RouteAgent, RouteRequest};
use yew::Bridged;
use yew::{
    html,
    virtual_dom::VNode,
    Bridge, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use crate::YewRouterState;
use log::{warn, trace};
use yew::html::{ChildrenWithProps, ChildrenRenderer};
use crate::component_router::route::Route;
use yew::virtual_dom::VChild;


/// Rendering control flow component.
///
/// Based on the current url and its child [Routes](struct.Route.html), it will choose one route and
/// render its associated component.
///
///
/// # Example
/// ```
/// use yew::prelude::*;
/// use yew_router::{Router, Route, route, FromMatches};
///
/// pub struct AComponent {}
///
/// #[derive(Properties, FromMatches)]
/// pub struct AComponentProps {
///     value: String
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
///                 <Route path=route!("/a/{value}" => AComponent ) />
///             </Router>
///         }
///     }
/// }
/// ```
pub struct Router<T: for<'de> YewRouterState<'de>> {
    route: RouteInfo<T>,
    props: Props<T>,
    router_agent: Box<dyn Bridge<RouteAgent<T>>>,
}

/// Message for Router.
pub enum Msg<T> {
    UpdateRoute(RouteInfo<T>),
}

// TODO this may be the key to something
pub type RouteChilds<PAR> = ChildrenRenderer<VChild<Box<dyn From<String>>, PAR>>;

/// Properties for Router.
#[derive(Properties)]
pub struct Props<T: for<'de> YewRouterState<'de>> {
    #[props(required)]
    children: ChildrenWithProps<Route<T>, Router<T>>
}

impl <T> Component for Router<T>
    where T: for<'de> YewRouterState<'de>
{
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let mut router_agent = RouteAgent::bridge(callback);

        router_agent.send(RouteRequest::GetCurrentRoute);
        Router {
            route: Default::default(), // This must be updated by immediately requesting a route update from the service bridge.
            props,
            router_agent,
        }
    }

//    fn mounted(&mut self) -> ShouldRender {
//        self.router_agent.send(RouterRequest::GetCurrentRoute);
//        false
//    }

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

impl <T: for<'de> YewRouterState<'de>> Renderable<Router<T>> for Router<T>
{
    fn view(&self) -> VNode<Self> {

        trace!("Routing one of {} routes for  {:?}", self.props.children.iter().count(), &self.route);
        self.props.children.iter()
            .filter_map(|route| -> Option<Html<Self>> {
                route.props.path
                    .match_path(&self.route)
                    .map(|(_rest, hm)| {
                        let mut children = route.props.children.iter().peekable();

                        match (route.props.path.render_fn, children.peek()) {
                            (Some(render), Some(_)) => {
                                // If the component can't be created from the matches,
                                // the nested children will be rendered anyways
                                match (render)(&hm) {
                                    Some(rendered) => {
                                        Some(html!{
                                            <>
                                                {rendered}
                                                {for children}
                                            </>
                                        })
                                    }
                                    None => Some(html!{{for children}})
                                }
                            },
                            (Some(render), None)=> {
                                render(&hm)
                            }
                            (None, Some(_)) => Some(html!{{for children}}),
                            (None, None) => None
                        }
                    })
                    .ok()
                    .flatten_stable()
            })
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
