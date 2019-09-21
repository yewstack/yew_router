extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

mod from_captures;
use from_captures::from_captures_impl;

mod route;
use route::route_impl;

#[proc_macro_derive(FromCaptures)]
pub fn from_captures(tokens: TokenStream) -> TokenStream{
    from_captures_impl(tokens)
}

#[proc_macro_hack]
pub fn route(tokens: TokenStream) -> TokenStream {
    route_impl(tokens)
}



