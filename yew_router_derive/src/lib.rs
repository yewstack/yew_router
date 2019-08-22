use proc_macro_hack::proc_macro_hack;

// TODO the next logical step is to break PathMatcher into its own crate.
#[proc_macro_hack]
pub use yew_router_route_macro_impl::route; // TODO this is preventing a compile with error: undefined symbol: emscripten_asm_const_int

pub use yew_router_from_matches_macro_impl::FromMatches;
