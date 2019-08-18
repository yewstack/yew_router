
mod parser;
mod path_matcher;
pub use path_matcher::PathMatcher;
pub use path_matcher::OptimizedToken;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FromMatchesError {
    MissingField{ field_name: String},
    Error(Box<dyn Error>)
}

impl Display for FromMatchesError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            FromMatchesError::MissingField {field_name} => write!{f, "The field: '{}' was not present in your path matcher.", field_name},
            FromMatchesError::Error(e) => e.fmt(f)
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

pub trait FromMatches: Sized {
    fn from_matches(matches: &HashMap<String, String>) -> Result<Self, FromMatchesError>;
}

#[cfg(test)]
mod integration_test {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn literal_only() {
        let path_matcher = PathMatcher::try_from("/hello/there/general/kenobi").expect("Should parse");
        let (_, dict) = path_matcher.match_path("/hello/there/general/kenobi").expect("should match");
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn single_match_any_should_fail_to_match_over_separators() {
        let path_matcher = PathMatcher::try_from("/{test}/kenobi").expect("Should parse");
        path_matcher.match_path("/hello/there/general/kenobi").expect_err("should not match");
    }

    #[test]
    fn single_match_any_should_match_within_separator() {
        let path_matcher = PathMatcher::try_from("/*/kenobi").expect("Should parse");
        path_matcher.match_path("/hello/kenobi").expect("should match");
    }

    #[test]
    fn cant_capture_numeral_idents() {
        PathMatcher::try_from("/{3hello}").expect_err("Should not parse");
    }
}
