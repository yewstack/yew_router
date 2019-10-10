use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::IntoIter;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, Ident, Lit, Meta, MetaNameValue, Type, Variant,
};

const ATTRIBUTE_TOKEN_STRING: &str = "to";

pub fn switch_impl(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let enum_ident: Ident = input.ident;

    let variants: IntoIter<Variant> = match input.data {
        Data::Struct(_ds) => panic!("Deriving Switch not supported for Structs."),
        Data::Enum(de) => de.variants.into_iter(),
        Data::Union(_du) => panic!("Deriving FromCaptures not supported for Unions."),
    };

    let switch_variants: Vec<SwitchVariant> = variants
        .map(|variant: Variant| SwitchVariant {
            route_string: get_route_string(variant.attrs),
            ident: variant.ident,
            fields: variant.fields,
        })
        .collect();

    generate_trait_impl(enum_ident, switch_variants)
}

/// Gets this section:
/// `#[to = "/route/thing"]`
/// `       ^^^^^^^^^^^^^^`
/// After matching the "to".
fn get_route_string(attributes: Vec<Attribute>) -> String {
    attributes.iter()
        .filter_map(|attr: &Attribute| attr.parse_meta().ok())
        .filter_map(|meta: Meta| {
           match meta {
               Meta::NameValue(x) => Some(x),
               _ => None,
           }
       })
       .filter_map(|mnv: MetaNameValue| {
           mnv.path.clone()
               .get_ident()
               .filter(|ident| ident.to_string() == ATTRIBUTE_TOKEN_STRING.to_string())
               .map(move |_| {
                   match mnv.lit {
                       Lit::Str(s) => Some(s.value()),
                       _ => None
                   }
               })
               .flatten_stable()
       })
       .next()
       .unwrap_or_else(|| panic!(r##"The Switch derive expects all variants to be annotated with [{} = "/route/string"] "##, ATTRIBUTE_TOKEN_STRING))
}

pub struct SwitchVariant {
    route_string: String,
    ident: Ident,
    fields: Fields,
}

fn generate_trait_impl(enum_ident: Ident, switch_variants: Vec<SwitchVariant>) -> TokenStream {
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
            let SwitchVariant {
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
