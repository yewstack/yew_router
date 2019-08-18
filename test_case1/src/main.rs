use yew_router_route_macro_decl::route;
use route_info_parser::{PathMatcher, OptimizedToken};
use std::collections::HashMap;

fn main() {
    let pm = route!("/hello/{capture}");
    let hm: HashMap<String, String> = pm.match_path("/hello/there").expect("to match").1;
    assert_eq!(hm.get(&"capture".to_string()).unwrap(), &"there".to_string())
}
