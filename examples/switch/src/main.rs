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



    let route = RouteInfo::<()>::from("/single/32");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = RouteInfo::<()>::from("/othersingle/472");
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
    #[to = "{*:x}"]
    Single(Single),
    #[to = "{*:x}"]
    OtherSingle(OtherSingle)
}

#[derive(Switch, Debug)]
pub enum InnerRoute {
    #[to = "/left"]
    Left,
    #[to = "/right"]
    Right
}

#[derive(Switch, Debug)]
#[to = "/single/{number}"]
pub struct Single {
    number: u32
}

#[derive(Switch, Debug)]
#[to = "/othersingle/{number}"]
pub struct OtherSingle(u32);

