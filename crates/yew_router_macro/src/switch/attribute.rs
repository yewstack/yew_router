use crate::switch::shadow::{ShadowCapture, ShadowCaptureVariant, ShadowMatcherToken};
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

    /// The id is an unique identifier that allows otherwise unnamed captures to still be captured with unique names.
    pub fn into_shadow_matcher_tokens(
        self,
        id: usize,
        encountered_query: &mut bool,
    ) -> Vec<ShadowMatcherToken> {
        match self {
            AttrToken::To(matcher_string) => {
                yew_router_route_parser::parser::parse(&matcher_string)
                    .map(|tokens| yew_router_route_parser::optimize_tokens(tokens, false))
                    .expect("Invalid Matcher") // This is the point where users should see an error message if their matcher string has some syntax error.
                    .into_iter()
                    .map(crate::switch::shadow::ShadowMatcherToken::from)
                    .collect()
            }
            AttrToken::Lit(lit) => vec![ShadowMatcherToken::Exact(format!("/{}", lit))],
            AttrToken::Capture(Some(capture_name)) => vec![
                ShadowMatcherToken::Exact("/".to_string()),
                ShadowMatcherToken::Capture(ShadowCapture {
                    capture_variant: ShadowCaptureVariant::Named(capture_name),
                    allowed_captures: None,
                }),
            ],
            AttrToken::Capture(None) => vec![
                ShadowMatcherToken::Exact("/".to_string()),
                ShadowMatcherToken::Capture(ShadowCapture {
                    capture_variant: ShadowCaptureVariant::Named(id.to_string()),
                    allowed_captures: None,
                }),
            ],
            AttrToken::End => unimplemented!(
                "No matcher token currently exists for expressing the termination of a route"
            ),
            AttrToken::Rest(Some(capture_name)) => {
                vec![ShadowMatcherToken::Capture(ShadowCapture {
                    capture_variant: ShadowCaptureVariant::ManyNamed(capture_name),
                    allowed_captures: None,
                })]
            }
            AttrToken::Rest(None) => vec![ShadowMatcherToken::Capture(ShadowCapture {
                capture_variant: ShadowCaptureVariant::ManyNamed(id.to_string()),
                allowed_captures: None,
            })],
            AttrToken::Query(capture_name) => {
                if *encountered_query {
                    vec![
                        ShadowMatcherToken::Exact(format!("&{}=", capture_name)),
                        ShadowMatcherToken::Capture(ShadowCapture {
                            capture_variant: ShadowCaptureVariant::Named(capture_name),
                            allowed_captures: None,
                        }),
                    ]
                } else {
                    *encountered_query = true;
                    vec![
                        ShadowMatcherToken::Exact(format!("?{}=", capture_name)),
                        ShadowMatcherToken::Capture(ShadowCapture {
                            capture_variant: ShadowCaptureVariant::Named(capture_name),
                            allowed_captures: None,
                        }),
                    ]
                }
            }
            AttrToken::Frag(Some(capture_name)) => vec![
                ShadowMatcherToken::Exact("#".to_string()),
                ShadowMatcherToken::Capture(ShadowCapture {
                    capture_variant: ShadowCaptureVariant::Named(capture_name),
                    allowed_captures: None,
                }),
            ],
            AttrToken::Frag(None) => vec![
                ShadowMatcherToken::Exact("#".to_string()),
                ShadowMatcherToken::Capture(ShadowCapture {
                    capture_variant: ShadowCaptureVariant::Named(id.to_string()),
                    allowed_captures: None,
                }),
            ],
        }
    }
}
