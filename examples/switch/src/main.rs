fn main() {
    let route = Route::<()>::from("/some/route");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/some/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/another/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);


    let route = Route::<()>::from("/inner/left");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/yeet");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);
}
use yew_router::route::Route;
use yew_router::Switch;

#[derive(Switch, Debug)]
pub enum AppRoute {
    #[to = "/some/route"]
    SomeRoute,
    #[to = "/some/{thing}/{other}"]
    Something { thing: String, other: String},
    #[to = "/another/{thing}"]
    Another(String),
    #[to = "/inner{*:inner}"]
    Nested(InnerRoute),
}

#[derive(Switch, Debug)]
pub enum InnerRoute {
    #[to = "/left"]
    Left,
    #[to = "/right"]
    Right
}
