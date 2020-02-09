//! A component wrapping an `<a>` tag that changes the route.
use crate::{
    agent::{RouteAgentDispatcher, RouteRequest},
    route::Route,
    Switch,
};
use yew::prelude::*;

use super::{Msg, Props};
use crate::RouterState;
use cfg_if::cfg_if;
use yew::virtual_dom::VNode;

#[cfg(feature = "web_sys")]
use gloo::events::{EventListener, EventListenerOptions};
#[cfg(feature = "web_sys")]
use web_sys::HtmlLinkElement;

/// An anchor tag Component that when clicked, will navigate to the provided route.
///
/// Alias to RouterAnchor.
#[deprecated(note = "Has been renamed to RouterAnchor")]
pub type RouterLink<T> = RouterAnchor<T>;

/// An anchor tag Component that when clicked, will navigate to the provided route.
#[derive(Debug)]
pub struct RouterAnchor<SW: Switch + Clone + 'static, STATE: RouterState = ()> {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<STATE>,
    props: Props<SW>,
    #[cfg(feature = "web_sys")]
    a_ref: NodeRef,
    #[cfg(feature = "web_sys")]
    a_listener: Option<EventListener>,
}

impl<SW: Switch + Clone + 'static, STATE: RouterState> Component for RouterAnchor<SW, STATE> {
    type Message = Msg;
    type Properties = Props<SW>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        RouterAnchor {
            link,
            router,
            props,
            #[cfg(feature = "web_sys")]
            a_ref: NodeRef::default(),
            #[cfg(feature = "web_sys")]
            a_listener: None,
        }
    }

    #[cfg(feature = "web_sys")]
    fn mounted(&mut self) -> ShouldRender {
        if let Some(link) = self.a_ref.try_into::<HtmlLinkElement>() {
            let options = EventListenerOptions::enable_prevent_default();
            let callback = self.link.callback(|_| Msg::Clicked);

            let listener = EventListener::new_with_options(&link, "click", options, move |event| {
                event.stop_propagation();
                event.prevent_default();
                callback.emit(());
            });
            self.a_listener = Some(listener);
        }

        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                let route = Route::from(self.props.route.clone());
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
        #[cfg(feature = "std_web")]
        use stdweb::web::event::IEvent;

        let route: Route<STATE> = Route::from(self.props.route.clone());
        let target: &str = route.as_str();

        cfg_if! {
            if #[cfg(feature = "std_web")] {
                let cb = |x| self.link.callback(x);

                html! {
                    <a
                        class=self.props.classes.clone(),
                        onclick=cb(|event: ClickEvent| {
                            event.prevent_default();
                            Msg::Clicked
                        }),
                        disabled=self.props.disabled,
                        href=target,
                    >
                        {
                            #[allow(deprecated)]
                            &self.props.text
                        }
                        {self.props.children.iter().collect::<VNode>()}
                    </a>
                }
            } else if #[cfg(feature = "web_sys")] {
                html! {
                    <a
                        ref=self.a_ref.clone(),
                        class=self.props.classes.clone(),
                        disabled=self.props.disabled,
                        href=target,
                    >
                        {
                            #[allow(deprecated)]
                            &self.props.text
                        }
                        {self.props.children.iter().collect::<VNode>()}
                    </a>
                }
            }
        }
    }
}
