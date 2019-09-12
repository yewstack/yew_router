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

pub use parser::CaptureVariant;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
pub use token_optimizer::{
    next_delimiters, optimize_tokens, parse_str_and_optimize_tokens, MatcherToken,
};

/// An error type used when implementing `FromMatches`.
#[derive(Debug)]
pub enum FromMatchesError {
    /// Missing field
    MissingField {
        /// The name of the field expected to be present
        field_name: String,
    },
    /// Dynamic error
    Error(Box<dyn Error>),
    /// Unknown error
    UnknownErr, // TODO Will be removed soon. dyn error above needs to go, and replaced with the names of the failed type conversions.
}

impl Display for FromMatchesError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FromMatchesError::MissingField { field_name } => {
                write! {f, "The field: '{}' was not present in your path matcher.", field_name}
            }
            FromMatchesError::Error(e) => e.fmt(f),
            FromMatchesError::UnknownErr => write!(f, "unknown error"),
        }
    }
}

impl Error for FromMatchesError {
    //    fn source(&self) -> Option<&(dyn Error + 'static)> {
    //        match self  {
    //            FromMatchesError::MissingField {..} => None,
    //            FromMatchesError::Error(e) => Some(&e )
    //        }
    //    }
}

/// Used for constructing `Properties` from URL matches.
///
/// # Note
/// FromMatches, as derived, is pretty dumb and unreliable.
/// It is only suggested to derive FromMatches if the types in your struct are reliably convertible from `&str`.
/// In practice, this means that `String`, and the numeric types are safe bets.
///
/// The derive relies on [FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html) for converting types.
///
/// # Suggestions
/// * If you have one or more optional sections in your path matcher, you are best off implementing this yourself.
pub trait FromMatches: Sized {
    /// Produces the props from the hashmap.
    /// It is expected that `TryFrom<String>` be implemented on all of the types contained within the props.
    fn from_matches(matches: &HashMap<&str, String>) -> Result<Self, FromMatchesError>;
    /// Verifies that all of the field names produced by the PathMatcher exist on the target props.
    /// Should panic if not all match.
    /// Should only be used at compile time.
    fn verify(_field_names: &HashSet<String>) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;

    #[allow(unused)]
    #[derive(Debug)]
    struct TestStruct {
        hello: String,
        there: String,
        general: String,
        kenobi: String,
    }

    impl FromMatches for TestStruct {
        fn from_matches(matches: &HashMap<&str, String>) -> Result<Self, FromMatchesError> {
            let hello = matches
                .get("hello")
                .ok_or_else(|| FromMatchesError::MissingField {
                    field_name: "hello".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromMatchesError::UnknownErr)
                })?;

            let there = matches
                .get("there")
                .ok_or_else(|| FromMatchesError::MissingField {
                    field_name: "there".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromMatchesError::UnknownErr)
                })?;

            let general = matches
                .get("general")
                .ok_or_else(|| FromMatchesError::MissingField {
                    field_name: "general".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromMatchesError::UnknownErr)
                })?;

            let kenobi = matches
                .get("kenobi")
                .ok_or_else(|| FromMatchesError::MissingField {
                    field_name: "kenobi".to_string(),
                })
                .and_then(|m: &String| {
                    String::try_from(m.clone()).map_err(|_| FromMatchesError::UnknownErr)
                })?;

            let x = TestStruct {
                hello,
                there,
                general,
                kenobi,
            };
            Ok(x)
        }

        fn verify(field_names: &HashSet<String>) {
            if !field_names.contains(&"hello".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "hello".to_string()
                )
            }
            if !field_names.contains(&"there".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "there".to_string()
                )
            }
            if !field_names.contains(&"general".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "general".to_string()
                )
            }
            if !field_names.contains(&"kenobi".to_string()) {
                panic!(
                    "The struct expected the matches to contain a field named '{}'",
                    "kenobi".to_string()
                )
            }
        }
    }

    #[test]
    fn underived_verify_impl_is_valid() {
        let mut hs = HashSet::new();
        hs.insert("hello".to_string());
        hs.insert("there".to_string());
        hs.insert("general".to_string());
        hs.insert("kenobi".to_string());
        TestStruct::verify(&hs);
    }

    #[test]
    #[should_panic]
    fn underived_verify_impl_rejects_incomplete_matches() {
        let mut hs = HashSet::new();
        hs.insert("hello".to_string());
        hs.insert("there".to_string());
        hs.insert("general".to_string());
        TestStruct::verify(&hs);
    }

    #[test]
    fn underived_matches_impl_is_valid() {
        let mut hm = HashMap::new();
        hm.insert("hello", "You are".to_string());
        hm.insert("there", "a".to_string());
        hm.insert("general", "bold".to_string());
        hm.insert("kenobi", "one".to_string());
        TestStruct::from_matches(&hm).expect("should generate struct");
    }

    #[test]
    fn underived_matches_rejects_incomplete() {
        let mut hm = HashMap::new();
        hm.insert("hello", "You are".to_string());
        hm.insert("there", "a".to_string());
        hm.insert("general", "bold".to_string());
        TestStruct::from_matches(&hm).expect_err("should not generate struct");
    }
}

//#[cfg(test)]
//mod integration_test {
//    use super::*;
//    use std::convert::TryFrom;
//
//    #[test]
//    fn literal_only() {
//        let path_matcher = PathMatcher::try_from("/hello/there/general/kenobi").expect("Should parse");
//        let (_, dict) = path_matcher.match_path("/hello/there/general/kenobi").expect("should match");
//        assert_eq!(dict.len(), 0);
//    }
//
//    #[test]
//    fn single_match_any_should_fail_to_match_over_separators() {
//        let path_matcher = PathMatcher::try_from("/{test}/kenobi").expect("Should parse");
//        path_matcher.match_path("/hello/there/general/kenobi").expect_err("should not match");
//    }
//
//    #[test]
//    fn single_match_any_should_match_within_separator() {
//        let path_matcher = PathMatcher::try_from("/{}/kenobi").expect("Should parse");
//        path_matcher.match_path("/hello/kenobi").expect("should match");
//    }
//
//    #[test]
//    fn cant_capture_numeral_idents() {
//        PathMatcher::try_from("/{3hello}").expect_err("Should not parse");
//    }
//}
