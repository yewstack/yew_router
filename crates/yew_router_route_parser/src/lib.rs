//! Parser for a "matcher string". The tokens produced by this parser are used to construct a matcher.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_qualifications
)]

pub mod parser;
mod token_optimizer;

pub use parser::{Capture, CaptureVariant};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
pub use token_optimizer::{
    next_delimiters, optimize_tokens, parse_str_and_optimize_tokens, MatcherToken,
};

/// An error type used when implementing `FromCaptures`.
#[derive(Debug)]
pub enum FromCapturesError {
    /// Missing field
    MissingField {
        /// The name of the field expected to be present
        field_name: String,
    },
    /// Parsing the provided string failed.
    FailedParse {
        /// The name of the field that failed to parse.
        field_name: String,
        /// The source string from which the field should have been parsed.
        source_string: String,
    },
}

impl Display for FromCapturesError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FromCapturesError::MissingField { field_name } => write!(
                f,
                "The field: '{}' was not present in your path matcher.",
                field_name
            ),
            FromCapturesError::FailedParse {
                field_name,
                source_string,
            } => write!(
                f,
                "The field: `{}` was not able to be parsed from the provided string: `{}`.",
                field_name, source_string
            ),
        }
    }
}

impl Error for FromCapturesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Captures contain keys corresponding to named match sections,
/// and values containing the content captured by those sections.
pub type Captures<'a> = HashMap<&'a str, String>;

/// Used for constructing `Properties` from URL matches.
pub trait FromCaptures: Sized {
    /// Produces the props from the hashmap.
    /// It is expected that `TryFrom<String>` be implemented on all of the types contained within the props.
    fn from_captures(captures: &Captures) -> Result<Self, FromCapturesError>;
    /// Verifies that all of the field names produced by the PathMatcher exist on the target props.
    /// Should panic if not all match.
    /// Should only be used at compile time.
    fn verify(_field_names: &HashSet<String>) {}
}

impl FromCaptures for () {
    fn from_captures(_captures: &Captures) -> Result<Self, FromCapturesError> {
        Ok(())
    }
}

pub use captured_key_value::FromCapturedKeyValue;

/// Module for holding implementation details for the `FromCapturedKeyValue` trait.
mod captured_key_value {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::num::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    /// Some horrible hack to get around orphan rules so a `from_str` operation can be implemented on
    /// `Option` and `Result`.
    ///
    /// * The `Option` case will succeed if the item isn't in the `Captures` map, but will fail if it can't parse.
    /// * The `Result` case will succeed if the item can't be parsed, but will fail if it isn't present in the `Captures` map.
    /// * To cause the `FromCaptures::from_captures` derivation to never fail outright if either the item isn't present, nor formatted correctly, specify `Option<Result<T>>`.
    pub trait FromCapturedKeyValue: Sized {
        /// Reimplementation of `std::str::FromStr::from_str`, but returning an `Option` instead of a `Result`.
        fn from_value(s: &str) -> Option<Self>;
        /// If the key isn't available in the `Captures` map, the result of this function will be
        /// returned from the derived `FromCaptures::from_captures` function.
        fn key_not_available() -> Option<Self> {
            None // By default, capturing will fail.
        }
    }

    impl<T: FromCapturedKeyValue> FromCapturedKeyValue for Option<T> {
        fn from_value(s: &str) -> Option<Self> {
            Some(Some(FromCapturedKeyValue::from_value(s)?))
        }

        /// This will cause the derivation of `from_matches` to not fail if the key can't be located
        fn key_not_available() -> Option<Self> {
            Some(None)
        }
    }

    impl<T, E> FromCapturedKeyValue for Result<T, E>
    where
        T: FromStr<Err = E>,
    {
        fn from_value(s: &str) -> Option<Self> {
            Some(T::from_str(s))
        }
    }

    macro_rules! from_str_option_impl {
        ($SelfT: ty) => {
            impl FromCapturedKeyValue for $SelfT {
                fn from_value(s: &str) -> Option<Self> {
                    FromStr::from_str(s).ok()
                }
            }
        };
    }

    from_str_option_impl! {String}
    from_str_option_impl! {PathBuf}
    from_str_option_impl! {bool}

    from_str_option_impl! {IpAddr}
    from_str_option_impl! {Ipv4Addr}
    from_str_option_impl! {Ipv6Addr}
    from_str_option_impl! {SocketAddr}
    from_str_option_impl! {SocketAddrV4}
    from_str_option_impl! {SocketAddrV6}

    from_str_option_impl! {usize}
    from_str_option_impl! {u128}
    from_str_option_impl! {u64}
    from_str_option_impl! {u32}
    from_str_option_impl! {u16}
    from_str_option_impl! {u8}

    from_str_option_impl! {isize}
    from_str_option_impl! {i128}
    from_str_option_impl! {i64}
    from_str_option_impl! {i32}
    from_str_option_impl! {i16}
    from_str_option_impl! {i8}

    from_str_option_impl! {NonZeroU128}
    from_str_option_impl! {NonZeroU64}
    from_str_option_impl! {NonZeroU32}
    from_str_option_impl! {NonZeroU16}
    from_str_option_impl! {NonZeroU8}

    from_str_option_impl! {NonZeroI128}
    from_str_option_impl! {NonZeroI64}
    from_str_option_impl! {NonZeroI32}
    from_str_option_impl! {NonZeroI16}
    from_str_option_impl! {NonZeroI8}

    from_str_option_impl! {f64}
    from_str_option_impl! {f32}
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;

    #[allow(unused)]
    #[derive(Debug)]
    struct TestStruct {
        lorem: String,
        ipsum: String,
    }

    impl FromCaptures for TestStruct {
        fn from_captures(captures: &HashMap<&str, String>) -> Result<Self, FromCapturesError> {
            let lorem = captures
                .get("lorem")
                .ok_or_else(|| FromCapturesError::MissingField {
                    field_name: "lorem".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromCapturesError::FailedParse {
                        field_name: "lorem".to_string(),
                        source_string: m.to_string(),
                    })
                })?;

            let ipsum = captures
                .get("ipsum")
                .ok_or_else(|| FromCapturesError::MissingField {
                    field_name: "ipsum".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromCapturesError::FailedParse {
                        field_name: "ipsum".to_string(),
                        source_string: m.to_string(),
                    })
                })?;

            let x = TestStruct { lorem, ipsum };
            Ok(x)
        }

        fn verify(field_names: &HashSet<String>) {
            if !field_names.contains(&"lorem".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "lorem".to_string()
                )
            }
            if !field_names.contains(&"ipsum".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "ipsum".to_string()
                )
            }
        }
    }

    #[test]
    fn underived_verify_impl_is_valid() {
        let mut hs = HashSet::new();
        hs.insert("lorem".to_string());
        hs.insert("ipsum".to_string());
        TestStruct::verify(&hs);
    }

    #[test]
    #[should_panic]
    fn underived_verify_impl_rejects_incomplete_matches_lorem() {
        let mut hs = HashSet::new();
        hs.insert("lorem".to_string());
        TestStruct::verify(&hs);
    }

    #[test]
    #[should_panic]
    fn underived_verify_impl_rejects_incomplete_matches_ipsum() {
        let mut hs = HashSet::new();
        hs.insert("ipsum".to_string());
        TestStruct::verify(&hs);
    }

    #[test]
    fn underived_matches_impl_is_valid() {
        let mut hm = HashMap::new();
        hm.insert("lorem", "dolor".to_string());
        hm.insert("ipsum", "sit".to_string());
        TestStruct::from_captures(&hm).expect("should generate struct");
    }

    #[test]
    fn underived_matches_rejects_incomplete_lorem() {
        let mut hm = HashMap::new();
        hm.insert("lorem", "dolor".to_string());
        TestStruct::from_captures(&hm).expect_err("should not generate struct");
    }

    #[test]
    fn underived_matches_rejects_incomplete_ipsum() {
        let mut hm = HashMap::new();
        hm.insert("ipsum", "sit".to_string());
        TestStruct::from_captures(&hm).expect_err("should not generate struct");
    }

    #[test]
    fn error_display_missing_field() {
        let err = FromCapturesError::MissingField {
            field_name: "lorem".to_string(),
        };
        let displayed = format!("{}", err);
        let expected = "The field: 'lorem' was not present in your path matcher.";
        assert_eq!(displayed, expected);
    }

}
