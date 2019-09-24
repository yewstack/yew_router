use std::str::FromStr;
use std::usize;
use yew::prelude::*;
use yew::Properties;
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;
//use yew_router::FromCapturesError;
//use yew_router::{Captures, FromCaptures};
//use yew_router::{RouteAgent, RouteInfo};

pub struct BModel {
    props: Props,
    router: Box<dyn Bridge<RouteAgent>>,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[props(required)]
    number: Option<usize>,
    #[props(required)]
    sub_path: Option<String>,
}

pub enum Msg {
    Navigate(Vec<Msg>), // Navigate after performing other actions
    Increment,
    Decrement,
    UpdateSubpath(String),
    NoOp,
}

impl Component for BModel {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let callback = link.send_back(|_| Msg::NoOp);
        let router = RouteAgent::bridge(callback);

        BModel {
            props,
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Navigate(msgs) => {
                // Perform the wrapped updates first
                for msg in msgs {
                    self.update(msg);
                }

                // The path dictating that this component be instantiated must be provided
                let route_string = "/b".to_string();
                let route_string = match &self.props.sub_path {
                    Some(sub_path) => route_string + "?sub_path=" + &sub_path,
                    None => route_string,
                };
                let route_string = match &self.props.number.map(|x: usize| x.to_string()) {
                    Some(number) => route_string + "#" + &number,
                    None => route_string,
                };

                let route = RouteInfo::from(route_string);

                // Don't tell the router to alert its subscribers,
                // because the changes made here only affect the current component,
                // so mutation might as well be contained to the core component update loop
                // instead of being sent through the router.
                self.router
                    .send(RouteRequest::ChangeRouteNoBroadcast(route));
                true
            }
            Msg::NoOp => {
                false
            }
            Msg::Increment => {
                let n = if let Some(number) = self.props.number {
                    number + 1
                } else {
                    1
                };
                self.props.number = Some(n);
                true
            }
            Msg::Decrement => {
                let n: usize = if let Some(number) = self.props.number {
                    if number > 0 {
                        number - 1
                    } else {
                        number
                    }
                } else {
                    0
                };
                self.props.number = Some(n);
                true
            }
            Msg::UpdateSubpath(path) => {
                self.props.sub_path = Some(path);
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

impl FromCaptures for Props {
    fn from_captures(captures: &Captures) -> Result<Self, FromCapturesError> {
        let number = captures
            .get("number")
            .map(|n: &String| {
                usize::from_str(&n).map_err(|_| FromCapturesError::FailedParse {
                    field_name: "number".to_string(),
                    source_string: n.to_string(),
                })
            })
            .transpose()?;

        let props = Props {
            number,
            sub_path: captures.get("sub_path").cloned(),
        };
        Ok(props)
    }
}

impl BModel {
    fn display_number(&self) -> String {
        if let Some(number) = self.props.number {
            format!("Number: {}", number)
        } else {
            "Number: None".to_string()
        }
    }
    fn display_subpath_input(&self) -> Html<BModel> {
        let sub_path = self.props.sub_path.clone();
        html! {
            <input
                placeholder="subpath",
                value=sub_path.unwrap_or("".into()),
                oninput=|e| Msg::Navigate(vec![Msg::UpdateSubpath(e.value)]),
                />
        }
    }
}
