extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

mod from_matches;
use from_matches::from_matches_impl;

mod route;
use route::route_impl;

#[proc_macro_derive(FromMatches)]
pub fn from_matches(tokens: TokenStream) -> TokenStream{
    from_matches_impl(tokens)
}

#[proc_macro_hack]
pub fn route(tokens: TokenStream) -> TokenStream {
    route_impl(tokens)
}



