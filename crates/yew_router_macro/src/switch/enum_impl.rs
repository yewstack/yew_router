use crate::switch::{build_serializer_for_enum, SwitchItem, ImplSwitch};
use proc_macro2::Span;
use quote::quote;
use syn::{export::TokenStream2, Field, Fields, Generics, Ident, Type};
use syn::export::ToTokens;


pub struct EnumImpl {
    pub enum_ident: Ident,
    pub switch_variants: Vec<SwitchItem>,
    pub generics: Generics
}

impl ToTokens for EnumImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let variant_matchers = self.switch_variants.iter().map(|sv| {
            let SwitchItem {
                matcher,
                ident,
                fields,
            } = sv;
            let build_from_captures = build_variant_from_captures(&self.enum_ident, ident, fields);
            let matcher = super::build_matcher_from_tokens(&matcher);

            quote! {
                #matcher
                #build_from_captures
            }
        });

        let match_item = Ident::new("self", Span::call_site());
        let serializer = build_serializer_for_enum(&self.switch_variants, &self.enum_ident, &match_item);

        let inner = quote! {
            fn from_route_part<__T>(route: String, mut state: Option<__T>) -> (::std::option::Option<Self>, ::std::option::Option<__T>) {
                let route_string = route;
                #(#variant_matchers)*

                return (::std::option::Option::None, state)
            }

            fn build_route_section<__T>(self, mut buf: &mut ::std::string::String) -> ::std::option::Option<__T> {
                #serializer
            }
        };

        let token_stream = ImplSwitch {
            target_ident: &self.enum_ident,
            generics: &self.generics,
            inner
        }.to_token_stream();

        tokens.extend(token_stream)
    }
}


/// Once the 'captures' exists, attempt to populate the fields from the list of captures.
fn build_variant_from_captures(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
) -> TokenStream2 {
    match fields {
        Fields::Named(named_fields) => {
            let fields: Vec<TokenStream2> = named_fields
                .named
                .iter()
                .filter_map(|field: &Field| {
                    let field_ty: &Type = &field.ty;
                    field.ident.as_ref().map(|i: &Ident| {
                        let key = i.to_string();
                        (i, key, field_ty)
                    })
                })
                .map(|(field_name, key, field_ty): (&Ident, String, &Type)| {
                    quote! {
                        #field_name: {
                            let (v, s) = match captures.remove(#key) {
                                ::std::option::Option::Some(value) => {
                                    <#field_ty as ::yew_router::Switch>::from_route_part(
                                        value,
                                        state,
                                    )
                                }
                                ::std::option::Option::None => {
                                    (
                                        <#field_ty as ::yew_router::Switch>::key_not_available(),
                                        state,
                                    )
                                }
                            };
                            match v {
                                ::std::option::Option::Some(val) => {
                                    state = s; // Set state for the next var.
                                    val
                                },
                                ::std::option::Option::None => return (None, s) // Failed
                            }
                        }
                    }
                })
                .collect();

            quote! {
                let mut state = if let ::std::option::Option::Some(mut captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    let create_item = || {
                         (
                            ::std::option::Option::Some(
                                #enum_ident::#variant_ident {
                                    #(#fields),*
                                }
                            ),
                            state
                        )
                    };
                    let (val, state) = create_item();

                    if val.is_some() {
                        return (val, state);
                    }
                    state
                } else {
                    state
                };
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let fields = unnamed_fields.unnamed.iter().map(|f: &Field| {
                let field_ty = &f.ty;
                quote! {
                    {
                        let (v, s) = match drain.next() {
                            ::std::option::Option::Some(value) => {
                                <#field_ty as ::yew_router::Switch>::from_route_part(
                                    value,
                                    state,
                                )
                            },
                            ::std::option::Option::None => {
                                (
                                    <#field_ty as ::yew_router::Switch>::key_not_available(),
                                    state,
                                )
                            }
                        };
                        match v {
                            ::std::option::Option::Some(val) => {
                                state = s; // Set state for the next var.
                                val
                            },
                            ::std::option::Option::None => return (None, s) // Failed
                        }
                    }
                }
            });

            quote! {
                let mut state = if let ::std::option::Option::Some(mut captures) = matcher.capture_route_into_vec(&route_string).ok().map(|x| x.1) {
                    let mut drain = captures.drain(..);
                    let create_item = || {
                         (
                            ::std::option::Option::Some(
                                #enum_ident::#variant_ident(
                                    #(#fields),*
                                )
                            ),
                            state
                        )
                    };
                    let (val, state) = create_item();
                    if val.is_some() {
                        return (val, state);
                    }
                    state
                } else {
                    state
                };
            }
        }
        Fields::Unit => {
            quote! {
                let mut state = if let ::std::option::Option::Some(_captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    return (::std::option::Option::Some(#enum_ident::#variant_ident), state);
                } else {
                    state
                };
            }
        }
    }
}
