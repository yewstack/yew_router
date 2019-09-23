extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;

mod from_captures;
use from_captures::from_captures_impl;

mod route;
use route::route_impl;

/// Derives `FromCaptures` for the specified struct.
///
/// # Note
/// All fields must have types that implements `yew_router::matcher::FromCapturedKeyValue`.
///
/// # Examples
///
/// #### Simple
/// ```
/// use yew_router::{FromCaptures, Captures};
/// #[derive(FromCaptures)]
/// struct Test {
///     value: String,
///     number: u32,
/// }
/// let mut captures: Captures = Captures::new();
/// captures.insert("value", "SomeValue".to_string());
/// captures.insert("number", "5".to_string());
/// assert!(Test::from_captures(&captures).is_ok())
/// ```
///
/// #### Option and Result
/// ```
///# use std::num::ParseIntError;
/// use yew_router::{FromCaptures, Captures};
/// #[derive(FromCaptures)]
/// struct Test {
///     not_required: Option<String>,
///     parse_fail_allowed: Result<u32, ParseIntError>,
///     not_required_parse_fail_allowed: Option<Result<u32, ParseIntError>>
/// }
/// let mut captures: Captures = Captures::new();
/// captures.insert("parse_fail_allowed", "hello".to_string());
/// captures.insert("not_required_parse_fail_allowed", "hello".to_string());
/// assert!(Test::from_captures(&captures).is_ok())
/// ```
#[proc_macro_derive(FromCaptures)]
pub fn from_captures(tokens: TokenStream) -> TokenStream {
    from_captures_impl(tokens)
}

#[proc_macro_hack]
pub fn route(tokens: TokenStream) -> TokenStream {
    route_impl(tokens)
}
