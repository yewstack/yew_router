use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::parse_macro_input;
use syn::{Data, DeriveInput, Field, Fields, Ident};

pub fn from_captures_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let fields: Vec<Field> = match input.data {
        Data::Struct(ds) => {
            match ds.fields {
                Fields::Named(fields_named) => {
                    fields_named.named.iter().cloned().collect::<Vec<_>>()
                }
                Fields::Unnamed(_) => {
                    panic!("Deriving FromCaptures not supported for Tuple Structs.")
                }
                Fields::Unit => {
                    panic!("Deriving FromCaptures not supported for Unit Structs, but it should be in the near future. Open an issue .")
                }
            }

        }
        Data::Enum(_de) => {
            panic!("Deriving FromCaptures not supported for Enums.")
        }
        Data::Union(_du) => {
            panic!("Deriving FromCaptures not supported for Unions.")
        }
    };

    let keys = fields
        .iter()
        .cloned()
        .map(|f: Field| f.ident.unwrap())
        .map(|i: Ident| i.to_string())
        .collect::<Vec<_>>();

    let idents = fields.iter().cloned().map(|f: Field| f.ident.unwrap());
    let idents2 = idents.clone();
    let types = fields.iter().cloned().map(|f| f.ty);

    let assignments = quote! {
        #(
        let #idents = captures
            .get(#keys)
            .ok_or_else(|| {
                __FromCapturesError::MissingField {
                    field_name: #keys.to_string()
                }
            })
            .and_then(|m: &String| {
                let x: Result<#types, __FromCapturesError> = __FromStr::from_str(m.as_ref())
                    .map_err(|_| __FromCapturesError::UnknownErr);
                x
            })?;
        )*
    };

    let expanded = quote! {
            use yew_router::matcher::FromCapturesError as __FromCapturesError;
            use yew_router::matcher::FromCaptures as __FromCaptures;
            use std::collections::HashMap as __HashMap;
            use std::collections::HashSet as __HashSet;
            use std::str::FromStr as __FromStr;

            impl __FromCaptures for #name {
                fn from_captures(captures: &__HashMap<&str, String>) -> Result<Self, __FromCapturesError> {
                    #assignments

                    let x = #name {
                        #(#idents2),*
                    };
                    Ok(x)
                }

                fn verify(captures: &__HashSet<String>) {
                    #(
                        if !captures.contains(&#keys.to_string()) {
                            panic!("The struct expected the matches to contain a field named '{}'", #keys.to_string())
                        }
                    )*
                }
            }
        };
    TokenStream::from(expanded)
}