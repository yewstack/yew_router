use proc_macro2::{TokenStream};
use quote::quote;
use syn::export::ToTokens;


pub use self::build_route_section::BuildRouteSection;
pub use self::from_route_part::FromRoutePart;

mod build_route_section;
mod from_route_part;


pub struct InnerEnum<'a> {
    pub from_route_part: FromRoutePart<'a>,
    pub build_route_section: BuildRouteSection<'a>
}

impl <'a> ToTokens for InnerEnum<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let InnerEnum { from_route_part, build_route_section } = self;
        tokens.extend(quote! {
            #from_route_part
            #build_route_section
        });
    }
}

