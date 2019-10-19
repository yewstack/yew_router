//! A component wrapping a `<button>` tag that changes the route.
use crate::{
    agent::{RouteAgentDispatcher, RouteRequest},
    route::Route,
};
use yew::prelude::*;

use super::{Msg, Props};
use crate::RouterState;

/// Changes the route when clicked.
#[derive(Debug)]
pub struct RouterButton<T: for<'de> RouterState<'de>> {
    router: RouteAgentDispatcher<T>,
    props: Props<T>,
}

impl<T: for<'de> RouterState<'de>> Component for RouterButton<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterButton { router, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                let route = Route {
                    route: self.props.link.clone(),
                    state: self.props.state.clone(),
                };
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl<T: for<'de> RouterState<'de>> Renderable<RouterButton<T>> for RouterButton<T> {
    fn view(&self) -> Html<RouterButton<T>> {
        html! {
            <button
                class=self.props.classes.clone(),
                onclick=|_| Msg::Clicked,
                disabled=self.props.disabled,
            >
                {&self.props.text}
            </button>
        }
    }
}
