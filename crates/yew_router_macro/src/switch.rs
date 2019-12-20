use crate::switch::{
    shadow::{ShadowCaptureVariant, ShadowMatcherToken},
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    export::TokenStream2, parse_macro_input, Data, DeriveInput, Fields, GenericParam, Generics,
    Ident, Variant,
};

mod attribute;
mod enum_impl;
mod shadow;
mod struct_impl;

use self::attribute::AttrToken;
use syn::punctuated::Punctuated;
use yew_router_route_parser::FieldNamingScheme;
use crate::switch::struct_impl::{StructInner};
use crate::switch::enum_impl::{InnerEnum};

/// Holds data that is required to derive Switch for a struct or a single enum variant.
pub struct SwitchItem {
    pub matcher: Vec<ShadowMatcherToken>,
    pub ident: Ident,
    pub fields: Fields,
}

pub fn switch_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let ident: Ident = input.ident;
    let generics = input.generics;

    match input.data {
        Data::Struct(ds) => {
            let field_naming_scheme = match ds.fields {
                Fields::Unnamed(_) => FieldNamingScheme::Unnamed,
                Fields::Unit => FieldNamingScheme::Unit,
                Fields::Named(_) => FieldNamingScheme::Named,
            };
            let matcher = AttrToken::convert_attributes_to_tokens(input.attrs)
                .into_iter()
                .enumerate()
                .map(|(index, at)| at.into_shadow_matcher_tokens(index, field_naming_scheme))
                .flatten()
                .collect::<Vec<_>>();

            let item = SwitchItem {
                matcher,
                ident: ident.clone(), // TODO make SwitchItem take references instead.
                fields: ds.fields,
            };

            ImplSwitch {
                target_ident: &ident,
                generics: &generics,
                inner: StructInner {
                    from_route_part: struct_impl::FromRoutePart(&item),
                    build_route_section: struct_impl::BuildRouteSection {
                        switch_item: &item,
                        item: &Ident::new("self", Span::call_site())
                    }
                }
            }.to_token_stream().into()

        }
        Data::Enum(de) => {
            let switch_variants = de
                .variants
                .into_iter()
                .map(|variant: Variant| {
                    let field_type = match variant.fields {
                        Fields::Unnamed(_) => yew_router_route_parser::FieldNamingScheme::Unnamed,
                        Fields::Unit => FieldNamingScheme::Unit,
                        Fields::Named(_) => yew_router_route_parser::FieldNamingScheme::Named,
                    };
                    let matcher = AttrToken::convert_attributes_to_tokens(variant.attrs)
                        .into_iter()
                        .enumerate()
                        .map(|(index, at)| at.into_shadow_matcher_tokens(index, field_type))
                        .flatten()
                        .collect::<Vec<_>>();
                    SwitchItem {
                        matcher,
                        ident: variant.ident,
                        fields: variant.fields,
                    }
                })
                .collect::<Vec<SwitchItem>>();


            ImplSwitch {
                target_ident: &ident,
                generics: &generics,
                inner: InnerEnum {
                    from_route_part: enum_impl::FromRoutePart { switch_variants: &switch_variants, enum_ident: &ident },
                    build_route_section: enum_impl::BuildRouteSection {
                        switch_items: &switch_variants,
                        enum_ident: &ident,
                        match_item: &Ident::new("self", Span::call_site())
                    }
                }
            }.to_token_stream().into()
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
            case_insensitive: true,
        };
        let matcher = ::yew_router::matcher::RouteMatcher {
            tokens: ::std::vec![#(#tokens),*],
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
                write!(buf, "{}", #lit).unwrap();
            }
        }
        ShadowMatcherToken::Capture(capture) => match naming_scheme {
            FieldType::Named | FieldType::Unit => match &capture {
                ShadowCaptureVariant::Named(name)
                | ShadowCaptureVariant::ManyNamed(name)
                | ShadowCaptureVariant::NumberedNamed { name, .. } => {
                    let name = Ident::new(&name, Span::call_site());
                    quote! {
                        state = state.or(#name.build_route_section(buf));
                    }
                }
                ShadowCaptureVariant::Unnamed
                | ShadowCaptureVariant::ManyUnnamed
                | ShadowCaptureVariant::NumberedUnnamed { .. } => {
                    panic!("Unnamed matcher sections not allowed for named field types")
                }
            },
            FieldType::Unnamed { index } => {
                let name = unnamed_field_index_item(index);
                quote! {
                    state = state.or(#name.build_route_section(&mut buf));
                }
            }
        },
        ShadowMatcherToken::End => quote! {},
    }
}





/// Creates an ident used for destructuring unnamed fields.
///
/// There needs to be a unified way to "mangle" the unnamed fields so they can be destructured,
fn unnamed_field_index_item(index: usize) -> Ident {
    Ident::new(&format!("__field_{}", index), Span::call_site())
}

/// Creates the "impl <X,Y,Z> ::yew_router::Switch for TypeName<X,Y,Z> where etc.." line.
pub struct ImplSwitch<'a, T: ToTokens> {
    target_ident: &'a Ident,
    generics: &'a Generics,
    inner: T
}

impl <'a, T: ToTokens> ToTokens for ImplSwitch<'a, T> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let ident = self.target_ident;
        let inner = &self.inner;

        let line_tokens = if self.generics.params.is_empty() {
            quote! {
                impl ::yew_router::Switch for #ident {
                    #inner
                }
            }
        } else {
            let params = &self.generics.params;
            let param_idents = params
                .iter()
                .map(|p: &GenericParam| {
                    match p {
                        GenericParam::Type(ty) => ty.ident.clone(),
//                    GenericParam::Lifetime(lt) => lt.lifetime, // TODO different type here, must be handled by collecting into a new enum and defining how to convert _that_ to tokens.
                        _ => unimplemented!("Not all type parameter variants (lifetimes and consts) are supported in Switch")
                    }
                })
                .collect::<Punctuated<_,syn::token::Comma>>();

            let where_clause = &self.generics.where_clause;
            quote! {
                impl <#params> ::yew_router::Switch for #ident <#param_idents> #where_clause
                {
                    #inner
                }
            }
        };
        tokens.extend(line_tokens)
    }
}
