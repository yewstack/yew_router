use crate::switch::shadow::{ShadowCaptureVariant, ShadowMatcherToken};
use syn::{Attribute, Lit, Meta, MetaNameValue};

pub enum AttrToken {
    To(String),
    End,
    Rest(Option<String>),
}

impl AttrToken {
    pub fn convert_attributes_to_tokens(attributes: Vec<Attribute>) -> Vec<Self> {
        fn get_meta_name_value_str(mnv: &MetaNameValue) -> Option<String> {
            match &mnv.lit {
                Lit::Str(s) => Some(s.value()),
                _ => None,
            }
        }

        attributes
            .iter()
            .filter_map(|attr: &Attribute| attr.parse_meta().ok())
            .filter_map(|meta: Meta| match meta {
                Meta::NameValue(mnv) => mnv
                    .path
                    .clone()
                    .get_ident()
                    .into_iter()
                    .filter_map(|ident| match ident.to_string().as_str() {
                        "to" => Some(AttrToken::To(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `to` must be a String"),
                        )),
                        "rest" => Some(AttrToken::Rest(Some(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `rest` must be a String"),
                        ))),
                        _ => None,
                    })
                    .next(),
                Meta::Path(path) => path
                    .get_ident()
                    .into_iter()
                    .filter_map(|ident| match ident.to_string().as_str() {
                        "end" => Some(AttrToken::End),
                        "rest" => Some(AttrToken::Rest(None)),
                        _ => None,
                    })
                    .next(),
                _ => None,
            })
            .collect()
    }

    /// The id is an unique identifier that allows otherwise unnamed captures to still be captured
    /// with unique names.
    pub fn into_shadow_matcher_tokens(self, id: usize) -> Vec<ShadowMatcherToken> {
        match self {
            AttrToken::To(matcher_string) => {
                yew_router_route_parser::parse_str_and_optimize_tokens(&matcher_string)
                    .expect("Invalid Matcher") // This is the point where users should see an error message if their matcher string has some syntax error.
                    .into_iter()
                    .map(crate::switch::shadow::ShadowMatcherToken::from)
                    .collect()
            }
            AttrToken::End => vec![ShadowMatcherToken::End],
            AttrToken::Rest(Some(capture_name)) => vec![ShadowMatcherToken::Capture(
                ShadowCaptureVariant::ManyNamed(capture_name),
            )],
            AttrToken::Rest(None) => vec![ShadowMatcherToken::Capture(
                ShadowCaptureVariant::ManyNamed(id.to_string()),
            )],
        }
    }
}
