use proc_macro::TokenStream;
//use proc_macro2::TokenStream as TokenStream2;
//use quote::quote;
use syn::{parse_macro_input, Fields};
//use syn::punctuated::IntoIter;
use crate::switch::enum_impl::generate_enum_impl;
use crate::switch::shadow::ShadowMatcherToken;
use crate::switch::struct_impl::generate_struct_impl;
use syn::export::TokenStream2;
use syn::{Data, DeriveInput, Ident, Variant};

mod attribute;
mod enum_impl;
mod shadow;
mod struct_impl;

use self::attribute::AttrToken;

/// Holds data that is required to derive Switch for a struct or a single enum variant.
pub struct SwitchItem {
    pub matcher: Vec<ShadowMatcherToken>,
    pub ident: Ident,
    pub fields: Fields,
}

pub fn switch_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let ident: Ident = input.ident;

    match input.data {
        Data::Struct(ds) => {
            let mut encountered_query = false;
            let matcher = AttrToken::convert_attributes_to_tokens(input.attrs)
                .into_iter()
                .enumerate()
                .map(|(index, at)| at.into_shadow_matcher_tokens(index, &mut encountered_query))
                .flatten()
                .collect::<Vec<_>>();
            let switch_item = SwitchItem {
                matcher,
                ident,
                fields: ds.fields,
            };
            generate_struct_impl(switch_item)
        }
        Data::Enum(de) => {
            let switch_variants = de.variants.into_iter().map(|variant: Variant| {
                let mut encountered_query = false;
                let matcher = AttrToken::convert_attributes_to_tokens(variant.attrs)
                    .into_iter()
                    .enumerate()
                    .map(|(index, at)| at.into_shadow_matcher_tokens(index, &mut encountered_query))
                    .flatten()
                    .collect::<Vec<_>>();
                SwitchItem {
                    matcher,
                    ident: variant.ident,
                    fields: variant.fields,
                }
            });
            generate_enum_impl(ident, switch_variants)
        }
        Data::Union(_du) => panic!("Deriving FromCaptures not supported for Unions."),
    }
}

trait Flatten<T> {
    /// Because flatten is a nightly feature. I'm making a new variant of the function here for stable use.
    /// The naming is changed to avoid this getting clobbered when object_flattening 60258 is stabilized.
    fn flatten_stable(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten_stable(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}

fn build_matcher_from_tokens(tokens: Vec<ShadowMatcherToken>) -> TokenStream2 {
    quote::quote! {
        let settings = ::yew_router::matcher::MatcherSettings {
            strict: true, // Don't add optional sections
            complete: false, // Allow incomplete matches. // TODO investigate if this is necessary here.
            case_insensitive: true,
        };
        let matcher = ::yew_router::matcher::RouteMatcher {
            tokens : vec![#(#tokens),*],
            settings
        };
    }
}
