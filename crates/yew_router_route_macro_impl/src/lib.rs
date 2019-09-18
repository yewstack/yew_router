extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use std::hash::Hash;
use syn::export::TokenStream2;
use syn::parse::{Parse, ParseBuffer};
use syn::parse_macro_input;
use syn::Error;
use yew_router_route_parser::{CaptureVariant, MatcherToken};

struct S {
    /// The routing string
    s: String,
    case_insensitive: bool,
    incomplete: bool,
    strict: bool,
}

/// Custom keywords
mod kw {
    syn::custom_keyword!(CaseInsensitive);
    syn::custom_keyword!(Incomplete);
    syn::custom_keyword!(Strict);
}

/// Collects 0 or more results from the parse_options.
/// It prevents parsing the same token(s) (as specified in the vector of parsers) twice.
fn many_unordered<T: Eq + Hash>(
    input: &ParseBuffer,
    mut parse_options: Vec<&dyn Fn(&ParseBuffer) -> Option<T>>,
) -> HashSet<T> {
    let mut collected = HashSet::new();
    while !parse_options.is_empty() {
        let mut inserted = false;
        'x: for (index, f) in parse_options.iter().enumerate() {
            if let Some(keyword) = (f)(&input) {
                collected.insert(keyword);
                let _ = parse_options.remove(index);
                inserted = true;
                break 'x; // must break to make borrow checker approve the implicit mut borrow needed for remove.
            }
        }
        if !inserted {
            break;
        }
    }
    collected
}

impl Parse for S {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let s = input.parse::<syn::LitStr>()?;

        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
        enum Keyword {
            CaseInsensitive,
            Incomplete,
            Strict,
        }

        let collected: HashSet<Keyword> = many_unordered(
            input,
            vec![
                &|input| input.parse::<kw::Strict>().ok().map(|_| Keyword::Strict),
                &|input| {
                    input
                        .parse::<kw::Incomplete>()
                        .ok()
                        .map(|_| Keyword::Incomplete)
                },
                &|input| {
                    input
                        .parse::<kw::CaseInsensitive>()
                        .ok()
                        .map(|_| Keyword::CaseInsensitive)
                },
            ],
        );

        let incomplete = collected.contains(&Keyword::Incomplete);
        let strict = collected.contains(&Keyword::Strict);
        let case_insensitive = collected.contains(&Keyword::CaseInsensitive);

        Ok(S {
            s: s.value(),
            case_insensitive,
            incomplete,
            strict,
        })
    }
}

/// Expected to be used like: route!("/route/to/thing" => Component)
#[proc_macro_hack]
pub fn route(input: TokenStream) -> TokenStream {
    let s: S = parse_macro_input!(input as S);
    let input: String = s.s;

    // Do the parsing at compile time so the user knows if their matcher is malformed.
    // It will still be their responsibility to know that the corresponding Props can be acquired from a path matcher.
    let t = yew_router_route_parser::parse_str_and_optimize_tokens(input.as_str(), !s.strict)
        .expect("Invalid Path Matcher")
        .into_iter()
        .map(ShadowMatcherToken::from);

    let complete = !s.incomplete; // by default, complete is on.
    let strict = s.strict;
    let case_insensitive = s.case_insensitive;

    let expanded = quote! {
        {
            let settings = ::yew_router::matcher::route_matcher::MatcherSettings {
                strict: #strict,
                /// A matcher must consume all of the input to succeed.
                complete: #complete,
                /// All literal matches do not care about case.
                case_insensitive: #case_insensitive
            };
            ::yew_router::matcher::Matcher::from(
                ::yew_router::matcher::route_matcher::RouteMatcher {
                    tokens : vec![#(#t),*],
                    settings
                }
            )
        }
    };
    TokenStream::from(expanded)
}

impl ToTokens for ShadowMatcherToken {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use ShadowMatcherToken as SOT;
        let t: TokenStream2 = match self {
            SOT::Exact(s) => quote! {
                ::yew_router::matcher::MatcherToken::Exact(#s.to_string())
            },
            SOT::Capture(variant) => quote! {
                ::yew_router::matcher::MatcherToken::Capture(#variant)
            },
            SOT::Optional(optional) => quote! {
                ::yew_router::matcher::MatcherToken::Optional(vec![#(#optional),*])
            },
        };
        ts.extend(t)
    }
}

/// A shadow of the OptimizedToken type.
/// It should match it exactly so that this macro can expand to the original.
enum ShadowMatcherToken {
    Exact(String),
    Capture(ShadowCaptureVariant),
    Optional(Vec<ShadowMatcherToken>),
}

enum ShadowCaptureVariant {
    Unnamed,                                         // {} - matches anything
    ManyUnnamed,                                     // {*} - matches over multiple sections
    NumberedUnnamed { sections: usize },             // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed { sections: usize, name: String }, // {2:name} - captures a fixed number of sections with a given name.
}

impl ToTokens for ShadowCaptureVariant {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let t = match self {
            ShadowCaptureVariant::Unnamed => {
                quote! {::yew_router::matcher::CaptureVariant::Unnamed}
            }
            ShadowCaptureVariant::ManyUnnamed => {
                quote! {::yew_router::matcher::CaptureVariant::ManyUnnamed}
            }
            ShadowCaptureVariant::NumberedUnnamed { sections } => {
                quote! {::yew_router::matcher::CaptureVariant::NumberedUnnamed{#sections}}
            }
            ShadowCaptureVariant::Named(name) => {
                quote! {::yew_router::matcher::CaptureVariant::Named(#name.to_string())}
            }
            ShadowCaptureVariant::ManyNamed(name) => {
                quote! {::yew_router::matcher::CaptureVariant::ManyNamed(#name.to_string())}
            }
            ShadowCaptureVariant::NumberedNamed { sections, name } => {
                quote! {::yew_router::matcher::CaptureVariant::NumberedNamed{#sections, #name.to_string()}}
            }
        };
        ts.extend(t)
    }
}

impl From<MatcherToken> for ShadowMatcherToken {
    fn from(ot: MatcherToken) -> Self {
        use MatcherToken as MT;
        use ShadowMatcherToken as SOT;
        match ot {
            MT::Exact(s) => SOT::Exact(s),
            MT::Capture(variant) => SOT::Capture(variant.into()),
            MT::Optional(optional) => SOT::Optional(optional.into_iter().map(SOT::from).collect()),
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
            CaptureVariant::NumberedUnnamed { sections } => SCV::NumberedUnnamed { sections },
            CaptureVariant::Named(name) => SCV::Named(name),
            CaptureVariant::ManyNamed(name) => SCV::ManyNamed(name),
            CaptureVariant::NumberedNamed { sections, name } => {
                SCV::NumberedNamed { sections, name }
            }
        }
    }
}
