use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub use yew_router_route_macro_impl::route; // This is preventing a compile with error: undefined symbol: emscripten_asm_const_int if it contains a dependency on yew

pub use yew_router_from_matches_macro_impl::FromMatches;
