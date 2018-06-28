//! Component that performs routing.

use yew::prelude::*;
use router::{RouteBase, Router};
use yew::html::Component;
use router::Request as RouterRequest;

use yew::virtual_dom::VNode;
use yew::virtual_dom::VList;
use yew::agent::Transferable;


use yew_patterns::{Sender, Receiver};

use component_routers::ComponentConstructorAttempter;
//use component_routers::ComponentWillTryToRoute;
use component_routers::ComponentResolverPackage;
use stdweb::JsSerialize;
use std::fmt::Debug;
use stdweb::unstable::TryFrom;
use stdweb::Value;
use serde::Serialize;
use serde::Deserialize;


/// Convenience alias for YewRouterBase.
/// If you don't store any state with the router, you should use this.
/// If you do need to store a state object with your routes, it is suggested that you define another
/// alias like `pub type MyRouter = YewRouterBase<MyState>` and use that around your project instead.
pub type YewRouter = YewRouterBase<()>;
pub type Props = PropsBase<()>;


pub enum Msg<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    SetRoute(RouteBase<T>),
//    SendRoutingFailure,
    ReceiveRoutingFailure,
    NoOp
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RoutingFailedMsg;

impl Transferable for RoutingFailedMsg {}

/// The role of the router.
enum RouterRole<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    /// If a router has a simple router role, it won't display an error message when it fails
    /// in routing one of its children.
    /// Instead, it will tell a PageNotFoundRouter via a RoutingFailedMsg channel to display its error page
    SimpleRouter(Sender<RoutingFailedMsg>),
    /// This router is configured to display a default page when a routing error occurs.
    /// Any SimpleRouter that encounters a routing error will alert this router type when it should
    /// display its 404 page.
    PageNotFoundRouter{
        /// RAII handle to the receiver that forwards external messages to the router
        _receiver: Receiver<RoutingFailedMsg>,
        /// The function that produces the default page when routing fails for any simple router.
        default_page: DefaultPage<T>,
        /// If this flag is set, the default page will be shown instead of one of the resolved routes.
        error_occurred: bool
    },
}

/// Properties of the router
#[derive(Clone, PartialEq, Default)]
pub struct PropsBase<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    /// A collection of functions that will facilitate route resolution and component construction.
    /// The easiest way to create this is to run the `routes![]` macro like so:
    /// `routes![ComponentName1, ComponentName2]`.
    pub routes: Vec<ComponentResolverPackage<T>>,
    /// The page that will be shown if any router can't resolve its route.
    pub page_not_found: Option<DefaultPage<T>>
}

pub struct YewRouterBase<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    /// Link for creating senders and receivers.
    link: ComponentLink<YewRouterBase<T>>,
    /// Bridge to the Router Agent. This will supply the YewRouter with messages related to the route.
    router: Box<Bridge<Router<T>>>,
    /// The current route.
    route: RouteBase<T>,
    /// The role of the YewRouter. If the YewRouter is constructed with a `page_not_found`,
    /// it will become a PageNotFoundRouter, which is capable of receiving notifications from
    /// SimpleRouters indicating that they failed to resolve their route. If no `page_not_found`
    /// is provided, the router becomes a SimpleRouter, and will alert above PageNotFoundRouters when
    /// it fails to route its children.
    role: RouterRole<T>,
    /// A collection of functions that facilitate route resolution and component construction.
    routes: Vec<ComponentResolverPackage<T>>,
}



#[derive(Clone)]
pub struct DefaultPage<T>(pub fn(route: &RouteBase<T>) -> VNode<YewRouterBase<T>>)
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
;

impl <T> PartialEq for DefaultPage<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    fn eq(&self, other: &DefaultPage<T>) -> bool {
        // compare pointers // TODO investigate if this works?
        self.0 as *const () == other.0 as *const ()
    }
}
impl <T> Default for DefaultPage<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    fn default() -> Self {
        fn default_page_impl<U>(_route: &RouteBase<U>) -> VNode<YewRouterBase<U>>
            where for <'de> U: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
        {
            VNode::VList(VList::new())
        }
        DefaultPage(default_page_impl::<T>)
    }
}

impl <T> Component for YewRouterBase<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    type Message = Msg<T>;
    type Properties = PropsBase<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|route: RouteBase<T>| Msg::SetRoute(route));
        let router = Router::bridge(callback);
        // TODO Not sure if this is technically correct. This should be sent _after_ the component has been created.
        router.send(RouterRequest::GetCurrentRoute);

        // If the component is created with a page_not_found page,
        // then it needs to be able to receive messages telling it that another router failed.
        let role = if let Some(default_page) = props.page_not_found {
            let callback = link.send_back(|_| Msg::ReceiveRoutingFailure);
            RouterRole::PageNotFoundRouter {
                _receiver: Receiver::new(callback),
                default_page,
                error_occurred: false
            }
        } else {
            let callback = link.send_back(|_| Msg::NoOp);
            RouterRole::SimpleRouter(Sender::new(callback))
        };

        YewRouterBase {
            link,
            router,
            route: RouteBase::default(), // Empty route, may or may not match any possible routes. It should be quickly overwritten.
            role,
            routes: props.routes,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetRoute(route) => {
                self.route = route;

                if self.should_routing_error() {
                    match self.role {
                        // If the router isn't configured to display a 404 page,
                        // send a message to a router that is configured to display a 404 page.
                        RouterRole::SimpleRouter(ref sender) => sender.send(RoutingFailedMsg),
                        // If the router is configured to display a 404 page,
                        // just set the flag to display the 404 page.
                        RouterRole::PageNotFoundRouter{ref mut error_occurred, ..} => {
                            *error_occurred = true;
                        }
                    }
                } else {
                    if let RouterRole::PageNotFoundRouter {ref mut error_occurred, ..} = self.role {
                        *error_occurred = false
                    }
                }
                true
            }
            Msg::ReceiveRoutingFailure => {
                if let RouterRole::PageNotFoundRouter {ref mut error_occurred, ..} = self.role {
                    *error_occurred = true;
                    true
                } else {
                    unreachable!("The component should only receive routing failure messages when it has the PageNotFoundRouter role.")
                }
            }
            Msg::NoOp => false
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {

        match props.page_not_found {
            Some(new_default_page) => {
                if let RouterRole::PageNotFoundRouter {ref mut default_page, ..} = self.role {
                    default_page.0 = new_default_page.0;
                } else {
                    let callback = self.link.send_back(|_| Msg::ReceiveRoutingFailure);
                    let _receiver = Receiver::new(callback);
                    self.role = RouterRole::PageNotFoundRouter {default_page: new_default_page, _receiver, error_occurred: false}
                }
            }
            None => {
                if let RouterRole::PageNotFoundRouter{..} = self.role {
                    let callback = self.link.send_back(|_| Msg::NoOp);
                    self.role = RouterRole::SimpleRouter(Sender::new(callback))
                }

            }
        }

        self.routes = props.routes;
        true
    }
}

impl <T> YewRouterBase<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{

    /// Determines which child component to render based on the current route
    /// If none of the sub components can be rendered, return a default page or empty vdom node
    /// depending on the role of the router.
    fn resolve_child(&self) -> Html<YewRouterBase<T>> {

        if let RouterRole::PageNotFoundRouter {ref default_page, ref error_occurred, ..} = self.role {
            if *error_occurred {
                return (default_page.0)(&self.route)
            }
        }

        let routes_to_attempt: Vec<&ComponentConstructorAttempter<T>> = self.routes
            .iter()
            .filter_map(|resolver| {
                if (resolver.will_try_to_route.0)(&self.route) {
                    Some(&resolver.constructor_attempter)
                } else {
                    None
                }
            })
            .collect();

        for attempter in routes_to_attempt {
            if let Some(child) = (attempter.0)(&self.route) {
                return child
            }
        }

        if let RouterRole::PageNotFoundRouter{ref default_page, ..} = self.role {
            (default_page.0)(&self.route)
        } else {
            VNode::VList(VList::new()) // empty - no matched route
        }
    }

    /// If the routing is going to fail, send a message to any listening router so that it may
    /// display the failed route page.
    fn should_routing_error(&self) -> bool {
        let routes_to_attempt: Vec<&ComponentConstructorAttempter<T>> = self.routes
            .iter()
            .filter_map(|resolver| {
                if (resolver.will_try_to_route.0)(&self.route) {
                    Some(&resolver.constructor_attempter)
                } else {
                    None
                }
            })
            .collect();

        // If this is empty, then it shouldn't error, because the router being empty itself isn't a problem.
        if routes_to_attempt.len() == 0 {
            return false
        }

        // TODO for performance reasons, constructing the whole child component and then not mounting it is slow
        // TODO, instead just construct the Option<Props> then coerce to a bool
        // If the component is going to try to route, but ends up failing, then you should error
        for attempter in routes_to_attempt {
            if let Some(_child) = (attempter.0)(&self.route) {
                return false
            }
        }

        true
    }
}


impl <T> Renderable<YewRouterBase<T>> for YewRouterBase<T>
    where for <'de> T: JsSerialize + Clone + Debug + TryFrom<Value> + Default + Serialize + Deserialize<'de> + 'static + PartialEq
{
    fn view(&self) -> Html<YewRouterBase<T>> {

        self.resolve_child()
//        let inner: VNode<YewRouter> = self.resolve_child();
//        html!{
//            <div>
//                <div>
//                    {"ROUTER:"}
//                </div>
//                <div>
////                    {format!("active: {:?};  route: {:?}", self.resolver.is_some(), self.route,)}
//                </div>
//                {inner}
//            </div>
//        }
    }
}


/// Turns the provided component type name into wrapped functions that will create the component.
#[macro_export]
macro_rules! routes {
    ( $( $x:tt ), * ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                let v = $x::RESOLVER;
                temp_vec.push(v);
            )*
            temp_vec
        }
    };
}
