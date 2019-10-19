fn main() {
    let route = Route::<()>::from("/some/route");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/some/thing/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/another/other");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/inner/left");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/yeet"); // should not match
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/single/32");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let route = Route::<()>::from("/othersingle/472");
    let app_route = AppRoute::switch(route);
    dbg!(app_route);

    let mut buf = String::new();
    AppRoute::Another("yeet".to_string()).build_route_section::<()>(&mut buf);
    println!("{}", buf);

    let mut buf = String::new();
    AppRoute::Something {
        thing: "yeet".to_string(),
        other: "yote".to_string(),
    }
    .build_route_section::<()>(&mut buf);
    println!("{}", buf);

    let mut buf = String::new();
    OtherSingle(23).build_route_section::<()>(&mut buf);
    println!("{}", buf);
}
use yew_router::{route::Route, Switch};

#[derive(Debug, Switch)]
pub enum AppRoute {
    #[to = "/some/route"]
    SomeRoute,
    #[to = "/some/{thing}/{other}"]
    Something { thing: String, other: String },
    #[to = "/another/{thing}"]
    Another(String),
    #[to = "/doot/{one}/{two}"]
    Yeet(String, String),
    #[lit = "inner"]
    #[rest]
    Nested(InnerRoute),
    #[rest]
    Single(Single),
    #[rest]
    OtherSingle(OtherSingle),
}

#[derive(Switch, Debug)]
pub enum InnerRoute {
    #[lit = "left"] // same as #[to = "/left"]
    Left,
    #[to = "/right"]
    Right,
}

#[derive(Switch, Debug)]
#[to = "/single/{number}"]
pub struct Single {
    number: u32,
}

#[derive(Switch, Debug)]
#[to = "/othersingle/{number}"]
pub struct OtherSingle(u32);
