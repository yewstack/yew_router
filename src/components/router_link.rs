//! A component wrapping an `<a>` tag that changes the route.
use crate::{
    agent::{RouteAgentDispatcher, RouteRequest},
    route::Route,
};
use yew::prelude::*;

use super::{Msg, Props};
use crate::RouterState;
use yew::virtual_dom::VNode;

/// An anchor tag Component that when clicked, will navigate to the provided route.
#[derive(Debug)]
pub struct RouterLink<T: for<'de> RouterState<'de>> {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<T>,
    props: Props<T>,
}

impl<T: for<'de> RouterState<'de>> Component for RouterLink<T> {
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterLink {
            link,
            router,
            props,
        }
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

    fn view(&self) -> VNode {
        use stdweb::web::event::IEvent;
        let target: &str = &self.props.link;
        let cb = |x| self.link.callback(x);

        html! {
            <a
                class=self.props.classes.clone(),
                onclick=cb(|event: ClickEvent | {
                    event.prevent_default();
                    Msg::Clicked
                }),
                disabled=self.props.disabled,
                href=target,
            >
                {&self.props.text}
            </a>
        }
    }
}
