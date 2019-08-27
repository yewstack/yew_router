use nom::IResult;
use nom::sequence::{tuple};
use nom::combinator::{map, opt, all_consuming};
use nom::error::{ParseError, ErrorKind, VerboseError};
use nom::branch::alt;

mod core;
mod util;
mod path;
mod query;
mod fragment;

/// Tokens generated from parsing a path matcher string.
/// They will be optimized to another token type used to match URLs.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteParserToken {
    Separator,
    Match(String), // Any string
    Capture(CaptureVariant), // {_}
    QueryBegin, // ?
    QuerySeparator, // &
    QueryCapture{ident: String, capture_or_match: CaptureOrMatch}, // x=y
    FragmentBegin, // #
    Optional(Vec<RouteParserToken>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureVariant {
    Unnamed, // {} - matches anything
    ManyUnnamed, // {*} - matches over multiple sections
    NumberedUnnamed{sections: usize}, // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed{sections: usize, name: String} // {2:name} - captures a fixed number of sections with a given name.
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureOrMatch {
    Match(String),
    Capture(CaptureVariant)
}



#[derive(Debug, Clone)]
pub enum Error {
    Unspecified
}

impl ParseError<&str> for Error {
    fn from_error_kind(_input: &str, _kind: ErrorKind) -> Self {
        Error::Unspecified
    }

    fn append(_input: &str, _kind: ErrorKind, _other: Self) -> Self {
        Error::Unspecified
    }
}

pub fn parse(i: &str) -> Result<Vec<RouteParserToken>, nom::Err<(&str, ErrorKind)>> {
    alt((
        map(
            all_consuming(tuple(
                (
                    opt(path::path_parser),
                    opt(query::query_parser),
                    opt(fragment::fragment_parser)
                )
            )),
            |(path, query, fragment): (Option<Vec<RouteParserToken>>, Option<Vec<RouteParserToken>>, Option<Vec<RouteParserToken>>)| {
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
                tokens
            }
        ),
        map(core::capture, |t| vec![t])
    ))(i)
        .map(|(_, tokens)| tokens)
}



#[cfg(test)]
mod tests {
    use super::*;






    #[test]
    fn parse_can_handle_multiple_literals() {
        let parsed = parse("/hello/there").expect("should parse");
        assert_eq!(parsed.1, vec![RouteParserToken::Separator,
                                  RouteParserToken::Match("hello".to_string()),
                                  RouteParserToken::Separator,
                                  RouteParserToken::Match("there".to_string())]
        );
    }



    #[test]
    fn parse_can_handle_trailing_path_separator() {
        let parsed = parse("/hello/").expect("should parse");
        assert_eq!(parsed.1, vec![RouteParserToken::Separator,
                                  RouteParserToken::Match("hello".to_string()),
                                  RouteParserToken::Separator]
        );
    }

    #[test]
    fn parse_can_capture_section() {
        let parsed = parse("/hello/{there}").expect("should parse");
        assert_eq!(parsed.1, vec![
            RouteParserToken::Separator,
            RouteParserToken::Match("hello".to_string()),
            RouteParserToken::Separator,
            RouteParserToken::Capture(CaptureVariant::Named("there".to_string())),
        ]
        )
    }

    #[test]
    fn parse_can_handle_multiple_matches_per_section() {
        let parsed = parse("/hello/{there}general{}").expect("should parse");
        assert_eq!(parsed.1, vec![
            RouteParserToken::Separator,
            RouteParserToken::Match("hello".to_string()),
            RouteParserToken::Separator,
            RouteParserToken::Capture(CaptureVariant::Named("there".to_string())),
            RouteParserToken::Match("general".to_string()),
            RouteParserToken::Capture(CaptureVariant::Unnamed)
        ]
        )
    }



    #[test]
    fn parser_cant_contain_multiple_matches_in_a_row_0() {
        parse("/path*{match}").expect_err("Should not validate");
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
}