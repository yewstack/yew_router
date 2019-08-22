use crate::route::RouteInfo;
use crate::router_agent::{RouterAgent, RouterRequest};
use yew::Bridged;
use yew::{
    html,
    virtual_dom::VNode,
    Bridge, Component, ComponentLink, Html, Properties, Renderable, ShouldRender,
};
use crate::YewRouterState;
use log::{warn, trace};
use yew_router_path_matcher::{PathMatcher};
use yew::html::ChildrenWithProps;


pub struct RouteChild {
    props: RouteChildProps
}

#[derive(Properties)]
pub struct RouteChildProps {
    #[props(required)]
    pub path: PathMatcher<Router>,
}

impl Component for RouteChild {
    type Message = ();
    type Properties = RouteChildProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        RouteChild {
            props
        }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

pub struct Router {
    route: RouteInfo<()>,
    props: Props,
    router_agent: Box<dyn Bridge<RouterAgent<()>>>,
}

pub enum Msg<T> {
    UpdateRoute(RouteInfo<T>),
}


#[derive(Properties)]
//pub struct Props<T: for<'de> YewRouterState<'de>> {
pub struct Props {
    #[props(required)]
    children: ChildrenWithProps<RouteChild, Router>
}

impl Component for Router {
    type Message = Msg<()>;
//    type Properties = Props<T>;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let mut router_agent = RouterAgent::bridge(callback);

        router_agent.send(RouterRequest::GetCurrentRoute);
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

impl Renderable<Router> for Router
{
    fn view(&self) -> VNode<Self> {
        let route : String = self.route.to_route_string();

        trace!("Routing one of {} routes for  {:?}", self.props.children.iter().count(), route);
        self.props.children.iter()
            .filter_map(|route_possibility| -> Option<Html<Self>> {
                route_possibility.props.path
                    .match_path(&route)
                    .map(|(_rest, hm)| {
                        (route_possibility.props.path.render_fn)(&hm)
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
