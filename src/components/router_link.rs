//! A component wrapping an `<a>` tag that changes the route.
use crate::{
    agent::{RouteAgentDispatcher, RouteRequest},
    route::Route,
};
use yew::prelude::*;

use super::{Msg, Props};
use crate::RouterState;

/// An anchor tag Component that when clicked, will navigate to the provided route.
#[derive(Debug)]
pub struct RouterLink<T: for<'de> RouterState<'de>> {
    router: RouteAgentDispatcher<T>,
    props: Props<T>,
}

impl<T: for<'de> RouterState<'de>> Component for RouterLink<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterLink { router, props }
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

impl<T: for<'de> RouterState<'de>> Renderable<RouterLink<T>> for RouterLink<T> {
    fn view(&self) -> Html<Self> {
        use stdweb::web::event::IEvent;
        let target: &str = &self.props.link;

        html! {
            <a
                class=self.props.classes.clone(),
                onclick=|event | {
                    event.prevent_default();
                    Msg::Clicked
                },
                disabled=self.props.disabled,
                href=target,
            >
                {&self.props.text}
            </a>
        }
    }
}
