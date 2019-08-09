use yew::{virtual_dom::{
    VNode,
    VComp,
    vcomp::ScopeHolder
}, Renderable, html, Html, Component, ComponentLink, ShouldRender, Properties, Bridge};
use route::RouteBase;
use router_agent::{RouterAgentBase, RouterRequest};
use serde::{Serialize, Deserialize};
use stdweb::{JsSerialize, Value};
use stdweb::unstable::TryFrom;
use std::fmt::Debug;
use yew::Bridged;

fn create_component<COMP: Component + Renderable<COMP>, CONTEXT: Component>(props: COMP::Properties) -> Html<CONTEXT> {
    let vcomp_scope: ScopeHolder<_> = Default::default(); // TODO, I don't exactly know what this does
    VNode::VComp(
        VComp::new::<COMP>(props, vcomp_scope)
    )
}

pub struct RouterOption<T, CONTEXT: Component> {
    optional_component_constructor: Box<dyn Fn(RouteBase<T>) -> Option<Html<CONTEXT>>>
}

impl <T, CONTEXT: Component> PartialEq for RouterOption<T, CONTEXT> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.optional_component_constructor.as_ref(), other.optional_component_constructor.as_ref())
    }
}

impl <T, CONTEXT: Component> RouterOption<T, CONTEXT> {

    pub fn new<COMP, F>(routing_condition: F) -> Self
    where
        COMP: Component + Renderable<COMP>,
        F: Fn(RouteBase<T>) -> Option<<COMP as Component>::Properties> + 'static,
    {
        RouterOption {
            optional_component_constructor: Box::new(
                move |route: RouteBase<T>| {
                    (routing_condition)(route)
                        .map(create_component::<COMP, CONTEXT>)
                }
            )
        }

    }
}


/// Implementation of the router "algorithm".
/// Routes the first option to succeed or if all fail, will display nothing and log an error.
fn route_one_of<CONTEXT: Component, T: Clone>(route_options: &[RouterOption<T, CONTEXT>], route: RouteBase<T>) -> Html<CONTEXT> {
    route_options
        .iter()
        .filter_map(|routing_option| (routing_option.optional_component_constructor)(route.clone()))
        .next()
        .unwrap_or_else(|| {
            error!("Routing failed. No default case was provided.");
            html!{ <></>}
        })
}

/// Router with state type of T
pub struct Router<T: Default + PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + JsSerialize + TryFrom<Value> + Debug + 'static> {
    route: RouteBase<T>,
    route_options: Vec<RouterOption<T, Router<T>>>,
    _router_agent: Box<dyn Bridge<RouterAgentBase<T>>>,
}

pub enum Msg<T> {
    UpdateRoute(RouteBase<T>),
}

#[derive(PartialEq, Properties)]
pub struct Props<T:  Default + PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + JsSerialize + TryFrom<Value> + Debug + 'static> {
    pub route_options: Vec<RouterOption<T, Router<T>>>
}

impl <T: Default + PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + JsSerialize + TryFrom<Value> + Debug + 'static> Component for Router<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let mut router_agent = RouterAgentBase::bridge(callback);
        // TODO Not sure if this is technically correct. This should be sent _after_ the component has been created.
        router_agent.send(RouterRequest::GetCurrentRoute);

        Router {
            route: Default::default(), // This must be updated by immediately requesting a route update from the service bridge.
            route_options: props.route_options,
            _router_agent: router_agent
        }
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
        self.route_options = props.route_options;
        true
    }

}
 impl <T: Default + PartialEq + Clone + Serialize + for<'de> Deserialize<'de> + JsSerialize + TryFrom<Value> + Debug + 'static> Renderable<Router<T>> for Router<T> {
     fn view(&self) -> VNode<Self> {
         route_one_of(&self.route_options, self.route.clone())
     }
 }
