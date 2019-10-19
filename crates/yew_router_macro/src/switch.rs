use proc_macro::TokenStream;
// use proc_macro2::TokenStream as TokenStream2;
// use quote::quote;
use syn::{parse_macro_input, Fields};
// use syn::punctuated::IntoIter;
use crate::switch::{
    enum_impl::generate_enum_impl,
    shadow::{ShadowCaptureVariant, ShadowMatcherToken},
    struct_impl::generate_struct_impl,
};
use proc_macro2::Span;
use quote::quote;
use syn::{export::TokenStream2, Data, DeriveInput, Ident, Variant};

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
            let switch_variants = de
                .variants
                .into_iter()
                .map(|variant: Variant| {
                    let mut encountered_query = false;
                    let matcher = AttrToken::convert_attributes_to_tokens(variant.attrs)
                        .into_iter()
                        .enumerate()
                        .map(|(index, at)| {
                            at.into_shadow_matcher_tokens(index, &mut encountered_query)
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    SwitchItem {
                        matcher,
                        ident: variant.ident,
                        fields: variant.fields,
                    }
                })
                .collect::<Vec<SwitchItem>>();
            generate_enum_impl(ident, switch_variants)
        }
        Data::Union(_du) => panic!("Deriving FromCaptures not supported for Unions."),
    }
}

trait Flatten<T> {
    /// Because flatten is a nightly feature. I'm making a new variant of the function here for
    /// stable use. The naming is changed to avoid this getting clobbered when object_flattening
    /// 60258 is stabilized.
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

fn build_matcher_from_tokens(tokens: &[ShadowMatcherToken]) -> TokenStream2 {
    quote! {
        let settings = ::yew_router::matcher::MatcherSettings {
            complete: false, // Allow incomplete matches. // TODO investigate if this is necessary here.
            case_insensitive: true,
        };
        let matcher = ::yew_router::matcher::RouteMatcher {
            tokens : vec![#(#tokens),*],
            settings
        };
    }
}

/// Enum indicating which sort of writer is needed.
pub(crate) enum FieldType {
    Named,
    Unnamed { index: usize },
    Unit,
}

/// This assumes that the variant/struct has been destructured.
fn write_for_token(token: &ShadowMatcherToken, naming_scheme: FieldType) -> TokenStream2 {
    match token {
        ShadowMatcherToken::Exact(lit) => {
            quote! {
                write!(buf, #lit).unwrap();
            }
        }
        ShadowMatcherToken::Capture(capture) => {
            match naming_scheme {
                FieldType::Named | FieldType::Unit => match &capture {
                    ShadowCaptureVariant::Named(name)
                    | ShadowCaptureVariant::ManyNamed(name)
                    | ShadowCaptureVariant::NumberedNamed { name, .. } => {
                        let name = Ident::new(&name, Span::call_site());
                        quote! {
                            state = state.or(#name.build_route_section(buf));
                        }
                    }
                },
                FieldType::Unnamed { index } => {
                    match &capture {
                        ShadowCaptureVariant::Named(_)
                        | ShadowCaptureVariant::ManyNamed(_)
                        | ShadowCaptureVariant::NumberedNamed { .. } => {
                            let name = unnamed_field_index_item(index);
                            // TODO this either needs to find type info from a ty passed in, or
                            // RouteInfo needs to be nixed.
                            quote! {
                                state = state.or(#name.build_route_section(&mut buf)); // TODO, this needs type information in order not to clobber the namespace. I don't want to have to import RouteInfo.
                            }
                        }
                    }
                }
            }
        }
        ShadowMatcherToken::End => quote!{}
    }
}

/// The serializer makes up the body of `build_route_section`.
pub fn build_serializer_for_enum(
    switch_items: &[SwitchItem],
    enum_ident: &Ident,
    match_item: &Ident,
) -> TokenStream2 {
    let variants = switch_items.iter().map(|switch_item: &SwitchItem| {
        let SwitchItem {
            matcher,
            ident,
            fields,
        } = switch_item;
        match fields {
            Fields::Named(fields_named) => {
                let field_names = fields_named
                    .named
                    .iter()
                    .filter_map(|named| named.ident.as_ref());
                let writers = matcher
                    .iter()
                    .map(|token| write_for_token(token, FieldType::Named));
                quote! {
                    #enum_ident::#ident{#(#field_names),*} => {
                        #(#writers)*
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let field_names = fields_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, _)| unnamed_field_index_item(index));
                let mut item_count = 0;
                let writers = matcher.iter().map(|token| {
                    if let ShadowMatcherToken::Capture(_) = &token {
                        let ts = write_for_token(token, FieldType::Unnamed { index: item_count });
                        item_count += 1;
                        ts
                    } else {
                        // Its either a literal, or something that will panic currently
                        write_for_token(token, FieldType::Unit)
                    }
                });
                quote! {
                    #enum_ident::#ident(#(#field_names),*) => {
                        #(#writers)*
                    }
                }
            }
            Fields::Unit => {
                let writers = matcher
                    .iter()
                    .map(|token| write_for_token(token, FieldType::Unit));
                quote! {
                    #enum_ident::#ident => {
                        #(#writers)*
                    }
                }
            }
        }
    });
    quote! {
        use ::std::fmt::Write as __Write; // TODO: is importing this here hygienic?
        let mut state: Option<T> = None;
        match #match_item {
            #(#variants)*,
        }
        return state;
    }
}

pub fn build_serializer_for_struct(switch_item: &SwitchItem, item: &Ident) -> TokenStream2 {
    let SwitchItem {
        matcher,
        ident,
        fields,
    } = switch_item;
    let destructor_and_writers = match fields {
        Fields::Named(fields_named) => {
            let field_names = fields_named
                .named
                .iter()
                .filter_map(|named| named.ident.as_ref());
            let writers = matcher
                .iter()
                .map(|token| write_for_token(token, FieldType::Named));
            quote! {
                let #ident{#(#field_names),*} = #item;
                #(#writers)*
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_names = fields_unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, _)| unnamed_field_index_item(index));
            let mut item_count = 0;
            let writers = matcher.iter().map(|token| {
                if let ShadowMatcherToken::Capture(_) = &token {
                    let ts = write_for_token(token, FieldType::Unnamed { index: item_count });
                    item_count += 1;
                    ts
                } else {
                    // Its either a literal, or something that will panic currently
                    write_for_token(token, FieldType::Unit)
                }
            });
            quote! {
                let #ident(#(#field_names),*) = #item;
                #(#writers)*
            }
        }
        Fields::Unit => {
            let writers = matcher
                .iter()
                .map(|token| write_for_token(token, FieldType::Unit));
            quote! {
                #(#writers)*
            }
        }
    };
    quote! {
        use ::std::fmt::Write as __Write; // TODO: is importing this here hygienic?
        let mut state: Option<T> = None;
        #destructor_and_writers
        return state;
    }
}

/// Creates an ident used for destructuring unnamed fields.
///
/// There needs to be a unified way to "mangle" the unnamed fields so they can be destructured,
fn unnamed_field_index_item(index: usize) -> Ident {
    Ident::new(&format!("__field_{}", index), Span::call_site())
}
