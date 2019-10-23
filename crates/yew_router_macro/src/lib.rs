extern crate proc_macro;
use proc_macro::TokenStream;

mod switch;

/// Implements `Switch` trait based on attributes present on the struct or enum variants.
#[proc_macro_derive(Switch, attributes(to, rest, end))]
pub fn switch(tokens: TokenStream) -> TokenStream {
    crate::switch::switch_impl(tokens)
}

#[proc_macro_attribute]
pub fn to(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn rest(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn end(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}
