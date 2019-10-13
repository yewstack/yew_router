extern crate proc_macro;
use proc_macro::TokenStream;

mod switch;


/// Implements `Switch` trait based on attributes present on the struct or enum variants.
#[proc_macro_derive(Switch, attributes(to, lit, cap, rest, query, frag))]
pub fn switch(tokens: TokenStream) -> TokenStream {
    crate::switch::switch_impl(tokens)
}

#[proc_macro_attribute]
pub fn to(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn lit(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn cap(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn rest(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn query(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn frag(_: TokenStream, _: TokenStream) -> TokenStream {
    TokenStream::new()
}
