use syn::{
     Field, Fields, Ident, Type,
};
use quote::quote;
use syn::export::TokenStream2;
use proc_macro::TokenStream;
use crate::switch::SwitchItem;


pub fn generate_enum_impl(enum_ident: Ident, switch_variants: impl Iterator<Item=SwitchItem>) -> TokenStream {
    /// Once the 'captures' exists, attempt to populate the fields from the list of captures.
    fn build_variant_from_captures(
        enum_ident: &Ident,
        variant_ident: Ident,
        fields: Fields,
    ) -> TokenStream2 {
        match fields {
            Fields::Named(named_fields) => {
                let fields: Vec<TokenStream2> = named_fields.named.into_iter()
                    .filter_map(|field: Field| {
                        let field_ty: Type = field.ty;
                        field.ident.map(|i| {
                            let key = i.to_string();
                            (i, key, field_ty)
                        })
                    })
                    .map(|(field_name, key, field_ty): (Ident, String, Type)|{
                        quote!{
                            #field_name: captures.get(#key) // TODO try to get an Option<T> instead of an Option<&T> out of the map.
                                .map_or_else(
                                    || <#field_ty as ::yew_router::Switch>::key_not_available(), // If the key isn't present, possibly resolve the case where the item is an option
                                    |value: &String| {
                                        <#field_ty as ::yew_router::Switch>::switch(::yew_router::route_info::RouteInfo{route: value.clone(), state})
                                    }
                                )?
                        }
                    })
                    .collect();

                quote! {
                    if let Some(captures) = matcher.capture_route_into_map(&route.to_string()).ok().map(|x| x.1) {
                        let produce_variant = move || -> Option<#enum_ident> {
                            Some(
                                #enum_ident::#variant_ident{
                                    #(#fields),*
                                }
                            )
                        };
                        if let Some(e) = produce_variant() {
                            return Some(e);
                        }
                    }
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                let fields = unnamed_fields.unnamed.iter()
                    .enumerate()
                    .map(|(index, f): (usize, &Field)|{
                        let field_ty = &f.ty;
                        quote!{
                            captures.get(#index)
                                .map_or_else(
                                    || <#field_ty as ::yew_router::Switch>::key_not_available(), // If the key isn't present, possibly resolve the case where the item is an option
                                    |(_key, value): &(&str, String)| {
                                        <#field_ty as ::yew_router::Switch>::switch(::yew_router::route_info::RouteInfo{route: value.clone(), state}) // TODO add the actual state here.
                                    }
                                )?
                        }
                    });

                quote! {
                    if let Some(captures) = matcher.capture_route_into_vec(&route.to_string()).ok().map(|x| x.1) {
                        let produce_variant = move || -> Option<#enum_ident> {
                            Some(
                                #enum_ident::#variant_ident(
                                    #(#fields),*
                                )
                            )
                        };
                        if let Some(e) = produce_variant() {
                            return Some(e);
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    if let Some(captures) = matcher.capture_route_into_map(&route.to_string()).ok().map(|x| x.1) {
                        return Some(#enum_ident::#variant_ident);
                    }
                }
            }
        }
    }

    let variant_matchers: Vec<TokenStream2> = switch_variants.into_iter()
        .map(|sv| {
            let SwitchItem {
                route_string, ident, fields
            } = sv;
            let build_from_captures = build_variant_from_captures(&enum_ident, ident, fields);

            quote! {
                let settings = ::yew_router::matcher::route_matcher::MatcherSettings {
                    strict: true, // Don't add optional sections
                    complete: false, // Allow incomplete matches. // TODO investigate if this is necessary here.
                    case_insensitive: true,
                };
                let matcher = ::yew_router::matcher::route_matcher::RouteMatcher::new(#route_string, settings)
                    .expect("Invalid Matcher");

                let state = route.state.clone(); // TODO State gets cloned a bunch here. Some refactorings should aim to remove this.
                #build_from_captures
            }
        })
        .collect::<Vec<_>>();

    let token_stream = quote! {
        impl ::yew_router::Switch for #enum_ident {
            fn switch<T: yew_router::route_info::RouteState>(route: ::yew_router::route_info::RouteInfo<T>) -> Option<Self> {
                #(#variant_matchers)*

                return None
            }
        }
    };
    TokenStream::from(token_stream)
}

