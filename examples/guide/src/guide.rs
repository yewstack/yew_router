
use yew::prelude::*;
use yew_router::{RouteAgent, RouteInfo};
use yew::html::ChildrenWithProps;
use yew::Properties;
use yew_router::path_matcher::PathMatcher;
use yew_router::components::RouterLink;
use yew_router::route_agent::RouteRequest::GetCurrentRoute;
use crate::page::{Page, PageProps};
use crate::markdown_window::MarkdownWindow;

pub struct Guide {
    router_agent: Box<dyn Bridge<RouteAgent>>,
    route: Option<RouteInfo>,
    props: GuideProps
}

#[derive(Properties)]
pub struct GuideProps {
    children: ChildrenWithProps<Page, Guide>
}

pub enum Msg {
    UpdateRoute(RouteInfo)
}

impl Component for Guide {
    type Message = Msg;
    type Properties = GuideProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(Msg::UpdateRoute);
        let router_agent = RouteAgent::bridge(callback);
        Guide {
            router_agent,
            route: None,
            props
        }
    }

    fn mounted(&mut self) -> ShouldRender {
        self.router_agent.send(GetCurrentRoute);
        false
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateRoute(route) => {
                self.route = Some(route);
            }
        }
        true
    }
}

impl Renderable<Guide> for Guide {
    fn view(&self) -> Html<Guide> {
        if let Some(route) = &self.route {
            let active_markdown_uri: Option<String> = self.props.children.iter()
                .filter_map(|child| {
                    if child.props.page_url == route.route {
                        Some(child.props.uri)
                    } else {
                        None
                    }
                })
                .next();

            let mut list_items = self.props.children
                .iter()
                .map(|child| {
                    render_page_list_item(child.props, route)
                });

            html! {
                <div style="">
                    <div style="">
                        <ul>
                            {for list_items}
                        </ul>
                    </div>
                    {
                        html !{
                            <MarkdownWindow uri=active_markdown_uri />
                        }
                    }
                </div>
            }
        } else {
            html! {}
        }
    }
}

fn render_page_list_item(props: PageProps, route: &RouteInfo) -> Html<Guide> {
    let pm: PathMatcher = PathMatcher::try_from(&props.page_url).unwrap();
    if pm.match_path(route).is_ok() {
        html! {
            <li>
                {"***"}
                <RouterLink link=props.page_url text={props.title} />
            </li>
        }
    } else {
        html! {
            <li>
                <RouterLink link=props.page_url text={props.title} />
            </li>
        }
    }
}