use yew::{Renderable, Component, Html, ComponentLink, ShouldRender};
use crate::route::Route;
use yew::html;
use serde::export::PhantomData;

pub struct RouterSwitch {
    components: Vec<()>
}


pub struct Props {
    pub routes: Vec<()> // TODO How will this be constructed? Another macro that extends the vec![]
}



enum RouteCase<T> {
    Default(T),
    Conditional()
}


pub struct Yeet {
    marker: PhantomData<Component>
}




/// Ideal Router
///
///
///```
///<Router>
///    <Route
///         path = {"/path/:id"}
///         exact
///         component = {Component1} // How would this be constrained to only allow Components with `Properties=RouterProps`?
///    >
///    <Route
///         path = {"/otherPath/:number"}
///         render = {|routerProps: | => html!{
///             <div> On page {routerProps.captures()["number"]} </div>
///         }}
///    />
///    <Route component = Component2 /> // Routes by default
///</Router>
///```
///
///
const YEET: u32 = 1;

//Box<dyn Component<Properties=(), Message=()>>
//
//pub trait RouteCase {
//    type Inner: Component;
//    fn show(&self, route: Route) -> Option<Html<Self::Inner>>;
//}
//
//
//impl <T: Component> RouteCase for T where T: Renderable<T> {
//    fn show(&self, _: Route) -> Option<Html<T>> {
//        Some(T::view(&self))
//    }
//}
//
//pub struct ConditionalRoute<T> {
//    component: Box<dyn Renderable<T>>,
//    condition: String // TODO make this a Regex
//}
//
//impl <T: Component + Renderable<T>> RouteCase for ConditionalRoute<T> {
//    fn show(&self, route: Route) -> Option<Html<ConditionalRoute<T>>> {
//        if route.to_route_string().eq(&self.condition) {
//            Some(html!{{T::view(&self)}})
//        } else {
//            None
//        }
//    }
//}
//
//pub enum Msg {}
//
//impl <T: Component + Renderable<Self>> Component for ConditionalRoute<T> {
//    // Some details omitted. Explore the examples to see more.
//
//    type Message = Msg;
//    type Properties = ();
//
//    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
//        unimplemented!()
//    }
//
//    fn update(&mut self, msg: Self::Message) -> ShouldRender {
//        true
//    }
//}
