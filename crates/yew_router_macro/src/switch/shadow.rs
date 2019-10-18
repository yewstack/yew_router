use quote::quote;
use quote::ToTokens;
use syn::export::TokenStream2;
use yew_router_route_parser::{CaptureVariant, MatcherToken};

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
        };
        ts.extend(t)
    }
}

/// A shadow of the OptimizedToken type.
/// It should match it exactly so that this macro can expand to the original.
pub enum ShadowMatcherToken {
    Exact(String),
    Capture(ShadowCaptureVariant),
}

pub enum ShadowCaptureVariant {
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed { sections: usize, name: String }, // {2:name} - captures a fixed number of sections with a given name.
}

impl ToTokens for ShadowCaptureVariant {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        let t = match self {
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
            MT::Capture(capture) => SOT::Capture(capture.into()),
        }
    }
}

impl From<CaptureVariant> for ShadowCaptureVariant {
    fn from(cv: CaptureVariant) -> Self {
        use ShadowCaptureVariant as SCV;
        match cv {
            CaptureVariant::Named(name) => SCV::Named(name),
            CaptureVariant::ManyNamed(name) => SCV::ManyNamed(name),
            CaptureVariant::NumberedNamed { sections, name } => {
                SCV::NumberedNamed { sections, name }
            }
        }
    }
}
