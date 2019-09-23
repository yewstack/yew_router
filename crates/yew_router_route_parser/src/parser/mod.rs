//! Parser that consumes a string and produces the first representation of the matcher.
use self::util::vectorize;
use nom::branch::alt;
use nom::combinator::{all_consuming, map_opt, opt};
use nom::error::{context, VerboseError};
use nom::sequence::tuple;

mod core;
mod error;
mod fragment;
mod path;
mod query;
pub mod util;

pub use error::YewRouterParseError;

/// Alias for a tuple of optional vectors of tokens representing the path, query, and fragment in that order.
type PathQueryFragmentTokens = (
    Option<Vec<RouteParserToken>>,
    Option<Vec<RouteParserToken>>,
    Option<Vec<RouteParserToken>>,
);

/// Tokens generated from parsing a route matcher string.
/// They will be optimized to another token type that is used to match URLs.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteParserToken {
    /// Match /
    Separator,
    /// Match a specific string.
    Exact(String),
    /// Match {_}. See `CaptureVariant` for more.
    Capture(CaptureVariant),
    /// Match ?
    QueryBegin,
    /// Match &
    QuerySeparator,
    /// Match x=y
    QueryCapture {
        /// Identifier
        ident: String,
        /// Capture or match
        capture_or_match: CaptureOrExact,
    },
    /// Match \#
    FragmentBegin,
    /// Optional section
    Optional(Vec<RouteParserToken>),
}

/// Token representing various types of captures.
///
/// It can capture and discard for unnamed variants, or capture and store in the `Matches` for the named variants.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureVariant {
    /// {} - matches anything.
    Unnamed,
    /// {*} - matches over multiple sections.
    ManyUnnamed,
    /// {4} - matches 4 sections.
    NumberedUnnamed {
        /// Number of sections to match.
        sections: usize,
    },
    /// {name} - captures a section and adds it to the map with a given name.
    Named(String),
    /// {*:name} - captures over many sections and adds it to the map with a given name.
    ManyNamed(String),
    /// {2:name} - captures a fixed number of sections with a given name.
    NumberedNamed {
        /// Number of sections to match.
        sections: usize,
        /// The key to be entered in the `Matches` map.
        name: String,
    },
}

/// Either a Capture, or an Exact match
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureOrExact {
    /// Match a specific string.
    Exact(String),
    /// Match a capture variant.
    Capture(CaptureVariant),
}

/// Parse "matcher string".
pub fn parse(i: &str) -> Result<Vec<RouteParserToken>, YewRouterParseError> {
    parse_impl(i).map_err(|error| {
        YewRouterParseError::from_err(i, error)
            .expect("Nom should always return an 'Error' from the parser.")
    })
}

/// Parse "matcher string" implementation.
pub fn parse_impl(i: &str) -> Result<Vec<RouteParserToken>, nom::Err<VerboseError<&str>>> {
    context(
        "parser",
        alt((
            all_consuming(vectorize(core::capture)), // TODO this should probably only be a subset of the normal capture. No {} or {named}
            all_consuming(map_opt(
                context(
                    "main matcher",
                    tuple((
                        opt(path::path_parser),
                        opt(query::query_parser),
                        opt(fragment::fragment_parser),
                    )),
                ),
                |(path, query, fragment): PathQueryFragmentTokens| {
                    let mut tokens = Vec::new();
                    if let Some(mut t) = path {
                        tokens.append(&mut t)
                    }
                    if let Some(mut t) = query {
                        tokens.append(&mut t)
                    }
                    if let Some(mut t) = fragment {
                        tokens.append(&mut t)
                    }
                    if !tokens.is_empty() {
                        Some(tokens)
                    } else {
                        None
                    }
                },
            )),
        )),
    )(i)
    .map(|(_, tokens)| tokens) // because of all_consuming, there should either be an error, or a success, no intermediate remaining input.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_can_handle_multiple_literals() {
        let parsed = parse("/hello/there").expect("should parse");
        assert_eq!(
            parsed,
            vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("hello".to_string()),
                RouteParserToken::Separator,
                RouteParserToken::Exact("there".to_string())
            ]
        );
    }

    #[test]
    fn parse_can_handle_trailing_path_separator() {
        let parsed = parse("/hello/").expect("should parse");
        assert_eq!(
            parsed,
            vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("hello".to_string()),
                RouteParserToken::Separator
            ]
        );
    }

    #[test]
    fn parse_can_capture_section() {
        let parsed = parse("/hello/{there}").expect("should parse");
        assert_eq!(
            parsed,
            vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("hello".to_string()),
                RouteParserToken::Separator,
                RouteParserToken::Capture(CaptureVariant::Named("there".to_string())),
            ]
        )
    }

    #[test]
    fn parse_can_handle_multiple_matches_per_section() {
        let parsed = parse("/hello/{there}general{}").expect("should parse");
        assert_eq!(
            parsed,
            vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("hello".to_string()),
                RouteParserToken::Separator,
                RouteParserToken::Capture(CaptureVariant::Named("there".to_string())),
                RouteParserToken::Exact("general".to_string()),
                RouteParserToken::Capture(CaptureVariant::Unnamed)
            ]
        )
    }

    #[test]
    fn parser_cant_contain_multiple_matches_in_a_row_0() {
        parse("/path{}{match}").expect_err("Should not validate");
    }
    #[test]
    fn parser_cant_contain_multiple_matches_in_a_row_1() {
        parse("/path{match1}{match2}").expect_err("Should not validate");
    }
    #[test]
    fn parser_cant_contain_multiple_matches_in_a_row_2() {
        parse("/path{}{}").expect_err("Should not validate");
    }

    #[test]
    fn parse_consumes_all_input() {
        parse("/hello/{").expect_err("Should not complete");
    }

    #[test]
    fn can_match_in_first_section() {
        parse("{any}").expect("Should validate");
    }

    #[test]
    fn can_match_in_first_section_1() {
        parse("{*}").expect("Should validate");
    }

    #[test]
    fn expected_slash_question_or_hash() {
        parse("hello").expect_err("Should not parse");
    }
}
