fn main() {
    let route = RouteInfo::<()>::from("/some/route");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/some/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/another/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/yeet");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);
}
use yew_router::route_info::RouteInfo;
use yew_router::Switch;

#[derive(Switch, Debug)]
pub enum AppRoute {
    #[to = "/some/route"]
    SomeRoute,
    #[to = "/some/{thing}"]
    Something { thing: String },
    #[to = "/another/{thing}"]
    Another(String),
}
