/// Generates a module named `router_state` containing aliases to common structures within yew_router
/// that deal with operating with RouteInfo and its state values as well as functions for
/// rendering routes.
///
/// Because they should be the same across a given application,
/// its a handy way to make sure that every type that could be needed is generated.
///
/// This macro is used to generate aliases and functions for the state type of `()` within yew_router.
/// Instead of doing these yourself, use this macro if you need to store state in the browser.
///
/// # Example
/// ```
///# #[macro_use] extern crate yew_router;
/// define_router_state!(Option<String>);
/// use router_state::Route; // alias to Route<Option<String>>
///# fn main() {}
/// ```
#[macro_export]
macro_rules! define_router_state {
    ($StateT:ty) => {
        define_router_state!($StateT, stringify!($StateT));
    };
    ($StateT:ty, $StateName:expr) => {
        #[doc = "A set of aliases to commonly used structures and functions used for routing."]
        mod router_state {

            #[doc = "The state that can be stored by the router service."]
            pub type State = $StateT;

            #[doc = "Alias to [RouteInfo<"]
            #[doc = $StateName]
            #[doc = ">](route_info/struct.RouteInfo.html)."]
            pub type RouteInfo = $crate::route_info::RouteInfo<$StateT>;

            #[doc = "Alias to [RouteService<"]
            #[doc = $StateName]
            #[doc = ">](route_service/struct.RouteService.html)."]
            pub type RouteService = $crate::route_service::RouteService<$StateT>;

            #[cfg(feature="router_agent")]
            #[doc = "Alias to [RouteAgent<"]
            #[doc = $StateName]
            #[doc = ">](agent/struct.RouteAgent.html)."]
            pub type RouteAgent = $crate::agent::RouteAgent<$StateT>;

            #[cfg(feature="router_agent")]
            #[doc = "Alias to [RouteAgentBridge<"]
            #[doc = $StateName]
            #[doc = ">](agent/bridge/struct.RouteAgentBridge.html)`."]
            pub type RouteAgentBridge = $crate::agent::bridge::RouteAgentBridge<$StateT>;

            #[cfg(feature="router")]
            #[doc = "Alias to [Router<"]
            #[doc = $StateName]
            #[doc = ">](router_component/router/struct.Router.html)."]
            pub type Router = $crate::router::Router<$StateT>;

            #[cfg(feature="router")]
            #[doc = "Alias to [Route<"]
            #[doc = $StateName]
            #[doc = ">](router_component/route/struct.Route.html)."]
            pub type Route = $crate::route::Route<$StateT>;

            #[cfg(feature="router")]
            #[doc = "Alias to [Render<"]
            #[doc = $StateName]
            #[doc = ">](router_component/render/struct.Render.html)."]
            pub type Render = $crate::render::Render<$StateT>;

            #[cfg(feature="components")]
            #[doc = "Alias to [RouteInjector<"]
            #[doc = $StateName]
            #[doc = ">](components/route_injector/struct.RouteInjector.html)."]
            pub type RouteInjector<C> = $crate::components::route_injector::RouteInjector<$StateT, C>;


            #[cfg(feature="router")]
            #[doc = "Renders the provided closure in terms of a `Router<"]
            #[doc = $StateName]
            #[doc = ">`."]
            pub fn render(render: impl $crate::render::RenderFn<Router> + 'static) -> $crate::render::Render<$StateT> {
                $crate::render::render(render)
            }

            #[cfg(feature="router")]
            #[doc = "Creates components using a Html block in terms of a `Router<"]
            #[doc = $StateName]
            #[doc = ">`."]
            #[doc = "\n"]
            #[doc = "Use a turbofish (`::<YourComponent>`) to indicate what component should be rendered."]
            pub fn component<T>() -> $crate::render::Render<$StateT>
            where
                T: yew::Component + yew::Renderable<T>,
                <T as yew::Component>::Properties: $crate::matcher::FromCaptures,
            {
                $crate::render::component::<T, $StateT>()
            }
        }
    }
}
