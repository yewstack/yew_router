// use crate::switch::{SwitchItem, write_for_token, FieldType, unnamed_field_index_item};
use crate::switch::SwitchItem;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Field, Fields, Type};
use crate::switch::attribute::get_attr_strings;


pub struct FromRoutePart<'a>(pub &'a SwitchItem);

impl<'a> ToTokens for FromRoutePart<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SwitchItem {
            matcher,
            ident,
            fields,
        } = &self.0;

        let matcher = super::super::build_matcher_from_tokens(&matcher);
        let build_from_captures = build_struct_from_captures(ident, fields);

        tokens.extend(quote! {
            fn from_route_part<__T>(
                route: String, mut state: Option<__T>
            ) -> (::std::option::Option<Self>, ::std::option::Option<__T>) {
                #matcher
                let route_string = route;

                #build_from_captures

                (::std::option::Option::None, state)
            }
        })
    }
}

fn build_struct_from_captures(ident: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named_fields) => {
            let fields: Vec<NamedField> = named_fields
                .named
                .iter()
                .filter_map(|field: &Field| {
                    let field_ty: &Type = &field.ty;
                    let is_state = get_attr_strings(field.attrs.clone()).any(|s| s.as_str() == "state");
                    field.ident.as_ref().map(|field_name| {
                        let key = field_name.to_string();
                        NamedField{field_name, key, field_ty, is_state}
                    })
                })
                .collect();

            return quote! {
                if let ::std::option::Option::Some(mut captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    return (
                        ::std::option::Option::Some(
                            #ident {
                                #(#fields),*
                            }
                        ),
                        state
                    );
                };
            };
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
                            ::std::option::Option::None => return (::std::option::Option::None, s) // Failed
                        }
                    }
                }
            });

            quote! {
                if let Some(mut captures) = matcher.capture_route_into_vec(&route_string).ok().map(|x| x.1) {
                    let mut drain = captures.drain(..);
                    return (
                        ::std::option::Option::Some(
                            #ident(
                                #(#fields),*
                            )
                        ),
                        state
                    );
                };
            }
        }
        Fields::Unit => {
            return quote! {
                let mut state = if let ::std::option::Option::Some(_captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    return (::std::option::Option::Some(#ident), state);
                } else {
                    state
                };
            }
        }
    }
}


pub struct NamedField<'a> {
    field_name: &'a Ident,
    key: String,
    field_ty: &'a Type,
    is_state: bool
}

impl <'a> ToTokens for NamedField<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let NamedField {
            field_name, key, field_ty, is_state
        } = self;
        // If the named field is marked as state, then just take the state,
        // otherwise try to match the key in the captures group
        if *is_state {
            tokens.extend(quote!{
                #field_name: {
                    state.take().expect("Can't take twice")
                }
            })
        } else {
            tokens.extend(quote! {
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
                        ::std::option::Option::None => return (::std::option::Option::None, s) // Failed
                    }
                }
            })
        }
    }
}