use yew_router::{route::Route, Switch};

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

    let route = Route::<()>::from("/option/test");
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
    #[to = "/inner"]
    #[rest]
    Nested(InnerRoute),
    #[rest] // Rest delegates the remaining input to the next attribute
    Single(Single),
    #[rest]
    OtherSingle(OtherSingle),
    /// Because this is an option, the inner item doesn't have to match.
    #[to = "/option/{thing}"]
    Optional(Option<String>),
    /// Because this is an option, a corresponding capture group doesn't need to exist
    #[to = "/missing/capture"]
    MissingCapture(Option<String>),
}

#[derive(Switch, Debug)]
pub enum InnerRoute {
    #[to = "/left"]
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

//#[derive(Switch, Debug)]
// pub enum Bad {
//    #[to = "/bad_route{4hello}"]
//    X,
//}
