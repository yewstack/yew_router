
use router;
use router::Route;
use yew::prelude::*;
use std::usize;

use yew_router::Routable;


pub struct BModel {
    number: Option<usize>,
    sub_path: Option<String>,
    router: Box<Bridge<router::Router<()>>>
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Props {
    number: Option<usize>,
    sub_path: Option<String>
}

pub enum Msg {
    Navigate(Vec<Msg>), // Navigate after performing other actions
    Increment,
    Decrement,
    UpdateSubpath(String),
    HandleRoute(Route)
}


impl Component for BModel {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let callback = link.send_back(|route: Route| Msg::HandleRoute(route));
        let router = router::Router::bridge(callback);

        router.send(router::Request::GetCurrentRoute);

        BModel {
            number: props.number,
            sub_path: props.sub_path,
            router
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Navigate(msgs) => {
                // Perform the wrapped updates first
                for msg in msgs{
                    self.update(msg);
                }

                // The path dictating that this component be instantiated must be provided
                let mut path_segments = vec!["b".into()];
                if let Some(ref sub_path) = self.sub_path {
                    path_segments.push(sub_path.clone())
                }

                let fragment: Option<String> = self.number.map(|x: usize | x.to_string());

                let route = router::Route {
                    path_segments,
                    query: None,
                    fragment,
                    state: (),
                };

                // Don't tell the router to alert its subscribers,
                // because the changes made here only affect the current component,
                // so mutation might as well be contained to the core component update loop
                // instead of being sent through the router.
                self.router.send(router::Request::ChangeRouteNoBroadcast(route));
                true
            }
            Msg::HandleRoute(route) => {
                // Instead of each component selecting which parts of the path are important to it,
                // it is also possible to match on the `route.to_route_string().as_str()` once
                // and create enum variants representing the different children and pass them as props.
                self.sub_path = route.path_segments.get(1).map(String::clone);
                self.number = route.fragment.and_then(|x| usize::from_str_radix(&x, 10).ok());

                true
            }
            Msg::Increment => {
                let n = if let Some(number) = self.number{
                    number + 1
                } else {
                    1
                };
                self.number = Some(n);
                true
            }
            Msg::Decrement => {
                let n: usize = if let Some(number) = self.number{
                    if number > 0 {
                        number - 1
                    } else {
                        number
                    }
                } else {
                    0
                };
                self.number = Some(n);
                true
            }
            Msg::UpdateSubpath(path) => {
                self.sub_path = Some(path);
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}


impl Renderable<BModel> for BModel {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <div>
                    { self.display_number() }
                    <button onclick=|_| Msg::Navigate(vec![Msg::Increment]),>{ "Increment" }</button>
                    <button onclick=|_| Msg::Navigate(vec![Msg::Decrement]),>{ "Decrement" }</button>
                </div>

                { self.display_subpath_input() }

            </div>
        }
    }
}

impl Routable for BModel {
    // Once proc macros land, it wouldn't be too difficult to set up a macro that does all of the below that looks like
    // #[route("/b/<sub_path>#<number>")]
    // That will implement this trait for the Component.
    //
    // The syntax could be extended to not care about prior paths like so:
    // #[route("/*/<sub_path>#<number>")]
    fn resolve_props(route: &router::Route) -> Option<Self::Properties> {
        let first_segment = route.path_segments.get(0).unwrap();
            if "b" == first_segment.as_str() {
                let mut props = Props {
                    number: None,
                    sub_path: None,
                };
                props.sub_path = route
                    .path_segments
                    .get(1)
                    .cloned();
                props.number = route
                    .clone()
                    .fragment
                    .and_then(|x: String| usize::from_str_radix(&x, 10).ok());
                Some(props)
            } else {
                None // This will only render if the first path segment is "b"
            }
    }

    fn will_try_to_route(route: &router::Route) -> bool {
        route.path_segments.get(0).is_some()
    }
}

impl BModel {
    fn display_number(&self) -> String {
        if let Some(number) = self.number {
            format!("Number: {}", number)
        } else {
            format!("Number: None")
        }
    }
    fn display_subpath_input(&self) -> Html<BModel> {
        let sub_path = self.sub_path.clone();
        html! {
            <input
                placeholder="subpath",
                value=sub_path.unwrap_or("".into()),
                oninput=|e| Msg::Navigate(vec![Msg::UpdateSubpath(e.value)]),
                />
        }
    }
}