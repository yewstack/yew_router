extern crate proc_macro;
use proc_macro::{TokenStream};
use yew_router_route_parser::{PathMatcher, OptimizedToken};
use std::convert::TryFrom;
use quote::{quote, ToTokens};
use syn::export::TokenStream2;
use proc_macro_hack::proc_macro_hack;
use syn::token::Token;
use syn::{Token, Error};
use syn::parse::{Parser, Parse, ParseBuffer};
use syn::parse_macro_input;

struct S {
    s: String
}
impl Parse for S {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let s = input.parse::<syn::LitStr>()?;
        Ok(S{s: s.value()})
    }
}

/// Expected to be used like: route!("/route/to/thing")
#[proc_macro_hack]
pub fn route(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as S);
    let s = s.s;

    // Do the parsing at compile time so the user knows if their matcher is malformed.
    // It will still be their responsibility to know that the corresponding Props can be acquired from a path matcher.
    let pm= PathMatcher::try_from(s.as_str()).expect("Invalid Path Matcher");
    let t = pm.tokens.into_iter().map(ShadowOptimizedToken::from);
    let expanded = quote!{
        PathMatcher {
            tokens : vec![#(#t),*]
        }
    };
    TokenStream::from(expanded)
}

impl ToTokens for ShadowOptimizedToken {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use ShadowOptimizedToken as SOT;
        let t: TokenStream2 = match self {
            SOT::Match(s) => {
                TokenStream2::from(quote!{OptimizedToken::Match(#s.to_string())})
            }
            SOT::MatchAny => {
                TokenStream2::from(quote!{OptimizedToken::MatchAny})
            }
            SOT::Capture { ident } => {
                let ident = ident.clone();
                TokenStream2::from(quote!{
                    OptimizedToken::Capture{ident: #ident.to_string()}
                })
            }
            SOT::QueryCapture { ident, value } => {
                let ident = ident.clone();
                let value = value.clone();
                TokenStream2::from(quote!{
                    OptimizedToken::QueryCapture{ident: #ident.to_string(), value: #value.to_string()}
                })
            }
        };
        ts.extend(t)
    }
}

/// A shadow of the OptimizedToken type.
/// It should match it exactly so that this macro can expand to the original.
enum ShadowOptimizedToken {
    Match(String),
    MatchAny,
    Capture{ ident: String},
    QueryCapture {
        ident: String,
        value: String
    }
}

impl From<OptimizedToken> for ShadowOptimizedToken {
    fn from(ot: OptimizedToken) -> Self {
        use OptimizedToken as OT;
        use ShadowOptimizedToken as SOT;
        match ot {
            OT::Match(s) => SOT::Match(s),
            OT::MatchAny => SOT::MatchAny,
            OT::Capture { ident} => SOT::Capture {ident},
            OT::QueryCapture { ident, value } => SOT::QueryCapture {ident, value}
        }
    }
}
