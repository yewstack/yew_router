use quote::quote;
use quote::ToTokens;
use syn::export::TokenStream2;
use yew_router_route_parser::{Capture, CaptureVariant, MatcherToken};

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
pub enum ShadowMatcherToken {
    Exact(String),
    Capture(ShadowCapture),
    Optional(Vec<ShadowMatcherToken>),
}

pub enum ShadowCaptureVariant {
    Unnamed,                                         // {} - matches anything
    ManyUnnamed,                                     // {*} - matches over multiple sections
    NumberedUnnamed { sections: usize },             // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed { sections: usize, name: String }, // {2:name} - captures a fixed number of sections with a given name.
}

pub struct ShadowCapture {
    pub capture_variant: ShadowCaptureVariant,
    pub allowed_captures: Option<Vec<String>>,
}

impl ToTokens for ShadowCapture {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ShadowCapture {
            capture_variant,
            allowed_captures,
        } = self;
        let t = match allowed_captures {
            Some(allowed_captures) => {
                quote! {
                    ::yew_router::matcher::Capture {
                        capture_variant: #capture_variant,
                        allowed_captures: vec![#(#allowed_captures),*]
                    }
                }
            }
            None => {
                quote! {
                    ::yew_router::matcher::Capture {
                        capture_variant: #capture_variant,
                        allowed_captures: None
                    }
                }
            }
        };
        tokens.extend(t)
    }
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
            MT::Capture(capture) => SOT::Capture(capture.into()),
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

impl From<Capture> for ShadowCapture {
    fn from(c: Capture) -> Self {
        ShadowCapture {
            capture_variant: c.capture_variant.into(),
            allowed_captures: c.allowed_captures,
        }
    }
}
