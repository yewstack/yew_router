extern crate proc_macro;
use proc_macro::{TokenStream};
use yew_router_route_parser::{MatcherToken, CaptureVariant};
use quote::{quote, ToTokens};
use syn::export::TokenStream2;
use proc_macro_hack::proc_macro_hack;
use syn::{Error};
use syn::parse::{Parse, ParseBuffer};
use syn::parse_macro_input;
//use syn::spanned::Spanned;

struct S {
    /// The routing string
    s: String,
}
impl Parse for S {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let s = input.parse::<syn::LitStr>()?;
        Ok(
            S {
                s: s.value(),
            }
        )
    }
}

/// Expected to be used like: route!("/route/to/thing" => Component)
#[proc_macro_hack]
pub fn route(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as S);
//    let target = s.target;
    let s: String = s.s;

    // Do the parsing at compile time so the user knows if their matcher is malformed.
    // It will still be their responsibility to know that the corresponding Props can be acquired from a path matcher.
    let t = yew_router_route_parser::parse_str_and_optimize_tokens(s.as_str())
        .expect("Invalid Path Matcher")
        .into_iter()
        .map(ShadowOptimizedToken::from);

    let expanded = quote!{
        {
            yew_router::path_matcher::PathMatcher {
                tokens : vec![#(#t),*],
            }
        }
    };
    TokenStream::from(expanded)
}

impl ToTokens for ShadowOptimizedToken {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use ShadowOptimizedToken as SOT;
        let t: TokenStream2 = match self {
            SOT::Match(s) => {
                TokenStream2::from(quote!{yew_router::path_matcher::MatcherToken::Match(#s.to_string())})
            }
            SOT::Capture ( variant ) => {
                TokenStream2::from(quote!{
                    yew_router::path_matcher::MatcherToken::Capture(#variant)
                })
            }
            SOT::Optional(optional) => {
                TokenStream2::from(quote!{
                    yew_router::path_matcher::MatcherToken::Optional(vec![#(#optional),*])
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
    Capture(ShadowCaptureVariant),
    Optional(Vec<ShadowOptimizedToken>)
}

enum ShadowCaptureVariant {
    Unnamed, // {} - matches anything
    ManyUnnamed, // {*} - matches over multiple sections
    NumberedUnnamed{sections: usize}, // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed{sections: usize, name: String} // {2:name} - captures a fixed number of sections with a given name.
}

impl ToTokens for ShadowCaptureVariant {

    fn to_tokens(&self, ts: &mut TokenStream2) {
        let t = match self {
            ShadowCaptureVariant::Unnamed => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::Unnamed}),
            ShadowCaptureVariant::ManyUnnamed => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::ManyUnnamed}),
            ShadowCaptureVariant::NumberedUnnamed { sections } => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::NumberedUnnamed{#sections}}),
            ShadowCaptureVariant::Named(name) => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::Named(#name.to_string())}),
            ShadowCaptureVariant::ManyNamed(name) => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::ManyNamed(#name.to_string())}),
            ShadowCaptureVariant::NumberedNamed { sections, name } => TokenStream2::from(quote!{yew_router::path_matcher::CaptureVariant::NumberedNamed{#sections, #name.to_string()}}),
        };
        ts.extend(t)

    }
}

impl From<MatcherToken> for ShadowOptimizedToken {
    fn from(ot: MatcherToken) -> Self {
        use MatcherToken as MT;
        use ShadowOptimizedToken as SOT;
        match ot {
            MT::Match(s) => SOT::Match(s),
            MT::Capture(variant) => SOT::Capture(variant.into()),
            MT::Optional(optional) => SOT::Optional(optional.into_iter().map(SOT::from).collect())
        }
    }
}

impl From<CaptureVariant> for ShadowCaptureVariant {

    fn from(cv: CaptureVariant) -> Self {
        use CaptureVariant as CV;
        use ShadowCaptureVariant as SCV;
        match cv {
            CV::Unnamed => SCV::Unnamed,
            CaptureVariant::ManyUnnamed => SCV::ManyUnnamed,
            CaptureVariant::NumberedUnnamed { sections } => SCV::NumberedUnnamed {sections},
            CaptureVariant::Named(name) => SCV::Named(name),
            CaptureVariant::ManyNamed(name) => SCV::ManyNamed(name),
            CaptureVariant::NumberedNamed { sections, name } => SCV::NumberedNamed {sections, name}
        }

    }
}
