use crate::switch::shadow::{ShadowCaptureVariant, ShadowMatcherToken};
use syn::{Attribute, Lit, Meta, MetaNameValue};

pub enum AttrToken {
    To(String),
    Lit(String),
    Capture(Option<String>),
    End,
    Rest(Option<String>),
    Query(String),
    Frag(Option<String>),
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
                        "lit" => Some(AttrToken::Lit(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `lit` must be a String`"),
                        )),
                        "capture" | "cap" => Some(AttrToken::Capture(Some(
                            get_meta_name_value_str(&mnv).expect(
                                "Value provided after `capture` or `cap` must be a String`",
                            ),
                        ))),
                        "rest" => Some(AttrToken::Rest(Some(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `rest` must be a String"),
                        ))),
                        "query" => Some(AttrToken::Query(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `rest` must be a String"),
                        )),
                        "frag" => Some(AttrToken::Frag(Some(
                            get_meta_name_value_str(&mnv)
                                .expect("Value provided after `frag` must be a String"),
                        ))),
                        _ => None,
                    })
                    .next(),
                Meta::Path(path) => path
                    .get_ident()
                    .into_iter()
                    .filter_map(|ident| match ident.to_string().as_str() {
                        "capture" | "cap" => Some(AttrToken::Capture(None)),
                        "end" => Some(AttrToken::End),
                        "rest" => Some(AttrToken::Rest(None)),
                        "frag" => Some(AttrToken::Frag(None)),
                        _ => None,
                    })
                    .next(),
                _ => None,
            })
            .collect()
    }

    /// The id is an unique identifier that allows otherwise unnamed captures to still be captured
    /// with unique names.
    pub fn into_shadow_matcher_tokens(
        self,
        id: usize,
        encountered_query: &mut bool,
    ) -> Vec<ShadowMatcherToken> {
        match self {
            AttrToken::To(matcher_string) => {
                yew_router_route_parser::parse_str_and_optimize_tokens(&matcher_string)
                    .expect("Invalid Matcher") // This is the point where users should see an error message if their matcher string has some syntax error.
                    .into_iter()
                    .map(crate::switch::shadow::ShadowMatcherToken::from)
                    .collect()
            }
            AttrToken::Lit(lit) => vec![ShadowMatcherToken::Exact(format!("/{}", lit))],
            AttrToken::Capture(Some(capture_name)) => vec![
                ShadowMatcherToken::Exact("/".to_string()),
                ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(capture_name)),
            ],
            AttrToken::Capture(None) => vec![
                ShadowMatcherToken::Exact("/".to_string()),
                ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(id.to_string())),
            ],
            AttrToken::End => unimplemented!(
                "No matcher token currently exists for expressing the termination of a route"
            ),
            AttrToken::Rest(Some(capture_name)) => vec![ShadowMatcherToken::Capture(
                ShadowCaptureVariant::ManyNamed(capture_name),
            )],
            AttrToken::Rest(None) => vec![ShadowMatcherToken::Capture(
                ShadowCaptureVariant::ManyNamed(id.to_string()),
            )],
            AttrToken::Query(capture_name) => {
                if *encountered_query {
                    vec![
                        ShadowMatcherToken::Exact(format!("&{}=", capture_name)),
                        ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(capture_name)),
                    ]
                } else {
                    *encountered_query = true;
                    vec![
                        ShadowMatcherToken::Exact(format!("?{}=", capture_name)),
                        ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(capture_name)),
                    ]
                }
            }
            AttrToken::Frag(Some(capture_name)) => vec![
                ShadowMatcherToken::Exact("#".to_string()),
                ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(capture_name)),
            ],
            AttrToken::Frag(None) => vec![
                ShadowMatcherToken::Exact("#".to_string()),
                ShadowMatcherToken::Capture(ShadowCaptureVariant::Named(id.to_string())),
            ],
        }
    }
}
