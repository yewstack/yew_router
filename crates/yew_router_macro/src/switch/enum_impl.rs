use crate::switch::{build_serializer_for_enum, SwitchItem};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::export::TokenStream2;
use syn::{Field, Fields, Ident, Type};

pub fn generate_enum_impl(enum_ident: Ident, switch_variants: Vec<SwitchItem>) -> TokenStream {
    let variant_matchers = switch_variants.iter().map(|sv| {
        let SwitchItem {
            matcher,
            ident,
            fields,
        } = sv;
        let build_from_captures = build_variant_from_captures(&enum_ident, ident, fields);
        let matcher = super::build_matcher_from_tokens(&matcher);

        quote! {
            #matcher
            #build_from_captures
        }
    });
    //        .collect::<Vec<_>>();

    let match_item = Ident::new("self", Span::call_site());
    let serializer = build_serializer_for_enum(&switch_variants, &enum_ident, &match_item);

    let token_stream = quote! {
        impl ::yew_router::Switch for #enum_ident {
            fn from_route_part<T: ::yew_router::route::RouteState>(route: ::yew_router::route::Route<T>) -> (Option<Self>, Option<T>) {
                let mut state = route.state;
                let route_string = route.route;
                #(#variant_matchers)*

                return (None, state)
            }

            fn build_route_section<T>(self, mut buf: &mut String) -> Option<T> {
                //pseudo-code:
                // For every field:
                //    write!(route, "{}", self.#field)
                // Return None for now, because marking routes isn't supported yet.
                #serializer
            }
        }
    };
    TokenStream::from(token_stream)
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
                                Some(value) => {
                                    <#field_ty as ::yew_router::Switch>::from_route_part(
                                        ::yew_router::route::Route {
                                            route: value,
                                            state,
                                        }
                                    )
                                }
                                None => {
                                    (
                                        <#field_ty as ::yew_router::Switch>::key_not_available(),
                                        state,
                                    )
                                }
                            };
                            match v {
                                Some(val) => {
                                    state = s; // Set state for the next var.
                                    val
                                },
                                None => return (None, s) // Failed
                            }
                        }
                    }
                })
                .collect();

            quote! {
                let mut state = if let Some(mut captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    let create_item = || {
                         (
                            Some(
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
                            Some((_key, value)) => {
                                <#field_ty as ::yew_router::Switch>::from_route_part(
                                    ::yew_router::route::Route {
                                        route: value,
                                        state,
                                    }
                                )
                            },
                            None => {
                                (
                                    <#field_ty as ::yew_router::Switch>::key_not_available(),
                                    state,
                                )
                            }
                        };
                        match v {
                            Some(val) => {
                                state = s; // Set state for the next var.
                                val
                            },
                            None => return (None, s) // Failed
                        }
                    }
                }
            });

            quote! {
                // TODO put an annotation here allowing unused muts.
                let mut state = if let Some(mut captures) = matcher.capture_route_into_vec(&route_string).ok().map(|x| x.1) {
                    let mut drain = captures.drain(..);
                    let create_item = || {
                         (
                            Some(
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
                let mut state = if let Some(_captures) = matcher.capture_route_into_map(&route_string).ok().map(|x| x.1) {
                    return (Some(#enum_ident::#variant_ident), state);
                } else {
                    state
                };
            }
        }
    }
}
