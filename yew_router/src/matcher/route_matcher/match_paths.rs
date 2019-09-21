use crate::matcher::route_matcher::util::tag_possibly_case_sensitive;
use crate::matcher::route_matcher::MatcherSettings;
use crate::matcher::Captures;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::sequence::terminated;
use nom::IResult;
use std::iter::Peekable;
use std::slice::Iter;
use yew_router_route_parser::parser::util::consume_until;
use yew_router_route_parser::{CaptureVariant, MatcherToken};

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn match_path<'a, 'b: 'a>(
    tokens: &'b [MatcherToken],
    settings: &'b MatcherSettings,
) -> impl Fn(&'a str) -> IResult<&'a str, Captures<'b>> {
    move |i: &str| match_path_impl(tokens, *settings, i)
}

/// TODO return a parser instead of the result so it can be made all_consuming.
fn match_path_impl<'a, 'b: 'a>(
    tokens: &'b [MatcherToken],
    settings: MatcherSettings,
    mut i: &'a str,
) -> IResult<&'a str, Captures<'b>> {
    trace!("Attempting to match path: {:?} using: {:?}", i, tokens);

    let mut iter = tokens.iter().peekable();

    let mut matches: Captures = Captures::new();

    while let Some(token) = iter.next() {
        i = match token {
            MatcherToken::Exact(literal) => {
                trace!("Matching '{}' against literal: '{}'", i, literal);
                tag_possibly_case_sensitive(literal.as_str(), !settings.case_insensitive)(i)?.0
            }
            MatcherToken::Optional(inner_tokens) => {
                match opt(|i| match_path_impl(&inner_tokens, settings, i))(i) {
                    Ok((ii, inner_matches)) => {
                        if let Some(inner_matches) = inner_matches {
                            matches.extend(inner_matches);
                        }
                        ii
                    }
                    _ => i, // Do nothing if this fails
                }
            }
            MatcherToken::Capture(variant) => match variant {
                CaptureVariant::Unnamed => capture_unnamed(i, &mut iter)?,
                CaptureVariant::ManyUnnamed => capture_many_unnamed(i, &mut iter)?,
                CaptureVariant::NumberedUnnamed { sections } => {
                    capture_numbered_unnamed(i, &mut iter, *sections)?
                }
                CaptureVariant::Named(name) => capture_named(i, &mut iter, &name, &mut matches)?,
                CaptureVariant::ManyNamed(name) => {
                    capture_many_named(i, &mut iter, &name, &mut matches)?
                }
                CaptureVariant::NumberedNamed { sections, name } => {
                    capture_numbered_named(i, &mut iter, &name, *sections, &mut matches)?
                }
            },
        };
    }
    debug!("Path Matched");

    Ok((i, matches))
}

// TODO see if _all_ of these outer if/else blocks could be removed.

/// Captures a section and doesn't add it to the matches.
///
/// It will capture characters until a separator or other invalid character is encountered
/// and the next string of characters is confirmed to be the next literal.
fn capture_unnamed<'a>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Unnamed");
    let ii = if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        alt((
            consume_until(alt((tag("/"), tag("?"), tag("#")))),
            consume_until(delimiter),
        ))(i)?
        .0
    } else if i.is_empty() {
        i // Match even if nothing is left
    } else {
        valid_capture_characters(i)?.0
    };
    Ok(ii)
}

fn capture_many_unnamed<'a>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    trace!("Matching ManyUnnamed");
    let ii = if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        consume_until(delimiter)(i)?.0
    } else if i.is_empty() {
        i // Match even if nothing is left
    } else {
        valid_many_capture_characters(i)?.0
    };
    Ok(ii)
}

fn capture_numbered_unnamed<'a>(
    mut i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    mut sections: usize,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedUnnamed ({})", sections);
    if let Some(_peaked_next_token) = iter.peek() {
        while sections > 0 {
            if sections > 1 {
                i = terminated(valid_capture_characters, tag("/"))(i)?.0;
            } else {
                // Don't consume the next character on the last section
                let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
                i = consume_until(delimiter)(i)?.0;
            }
            sections -= 1;
        }
    } else {
        while sections > 0 {
            if sections > 1 {
                i = terminated(valid_capture_characters, tag("/"))(i)?.0;
            } else {
                // Don't consume the next character on the last section
                i = valid_capture_characters(i)?.0;
            }
            sections -= 1;
        }
    }
    Ok(i)
}

fn capture_named<'a, 'b>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    capture_key: &'b str,
    matches: &mut Captures<'b>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Named ({})", capture_key);
    if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        let (ii, captured) = consume_until(delimiter)(i)?;
        matches.insert(capture_key, captured);
        Ok(ii)
    } else {
        let (ii, captured) = valid_capture_characters(i)?;
        matches.insert(capture_key, captured.to_string());
        Ok(ii)
    }
}

fn capture_many_named<'a, 'b>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    capture_key: &'b str,
    matches: &mut Captures<'b>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedUnnamed ({})", capture_key);
    if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        let (ii, captured) = consume_until(delimiter)(i)?;
        matches.insert(&capture_key, captured);
        Ok(ii)
    } else if i.is_empty() {
        matches.insert(&capture_key, "".to_string()); // TODO Is this a thing I want?
        Ok(i) // Match even if nothing is left
    } else {
        let (ii, c) = valid_many_capture_characters(i)?;
        matches.insert(&capture_key, c.to_string());
        Ok(ii)
    }
}

fn capture_numbered_named<'a, 'b>(
    mut i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    name: &'b str,
    mut sections: usize,
    matches: &mut Captures<'b>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedNamed ({}, {})", sections, name);
    let mut captured = "".to_string();
    if let Some(_peaked_next_token) = iter.peek() {
        while sections > 0 {
            if sections > 1 {
                let (ii, c) = terminated(valid_capture_characters, tag("/"))(i)?;
                i = ii;
                captured += c;
                captured += "/";
            } else {
                let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
                let (ii, c) = consume_until(delimiter)(i)?;
                i = ii;
                captured += &c;
            }
            sections -= 1;
            println!("{}", i);
        }
    } else {
        while sections > 0 {
            if sections > 1 {
                let (ii, c) = terminated(valid_capture_characters, tag("/"))(i)?;
                i = ii;
                captured += c;
            } else {
                // Don't consume the next character on the last section
                let (ii, c) = valid_capture_characters(i)?;
                i = ii;
                captured += c;
            }
            sections -= 1;
            println!("{}", i);
        }
    }
    matches.insert(&name, captured);
    Ok(i)
}

/// Characters that don't interfere with parsing logic for capturing characters
fn valid_capture_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " */#&?{}=";
    is_not(INVALID_CHARACTERS)(i)
}

fn valid_many_capture_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " #&?=";
    is_not(INVALID_CHARACTERS)(i)
}

//fn valid_capture_characters_in_query(i: &str) -> IResult<&str, &str> {
//    const INVALID_CHARACTERS: &str = " *#&?|{}=";
//    is_not(INVALID_CHARACTERS)(i)
//}

#[cfg(test)]
mod integration_test {
    use super::*;

    use yew_router_route_parser;
    //    use nom::combinator::all_consuming;

    #[test]
    fn match_query_after_path() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path?hello=there", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path?hello=there")
            .expect("should match");
    }

    #[test]
    fn match_query_after_path_trailing_slash() {
        let x =
            yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/?hello=there", true)
                .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path/?hello=there")
            .expect("should match");
    }

    // TODO this should be able to be less strict. A trailing slash before a # or ? should be ignored

    //    #[test]
    //    fn match_query_after_path_slash_ignored() {
    //        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/?hello=there").expect("Should parse");
    //        match_paths(&x, "/a/path?hello=there").expect("should match");
    //    }

    #[test]
    fn match_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?hello=there", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "?hello=there").expect("should match");
    }

    #[test]
    fn named_capture_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?hello={there}", true)
            .expect("Should parse");
        let (_, matches) =
            match_path_impl(&x, MatcherSettings::default(), "?hello=there").expect("should match");
        assert_eq!(matches["there"], "there".to_string())
    }

    #[test]
    fn match_n_paths_1() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*}", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/anything").expect("should match");
    }

    #[test]
    fn match_n_paths_2() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*}", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/anything/other/thing")
            .expect("should match");
    }

    #[test]
    fn match_n_paths_3() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*:cap}/thing", true)
            .expect("Should parse");
        let matches = match_path_impl(&x, MatcherSettings::default(), "/anything/other/thing")
            .expect("should match")
            .1;
        assert_eq!(matches["cap"], "anything/other".to_string())
    }

    #[test]
    fn match_n_paths_4() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*:cap}/thing", true)
            .expect("Should parse");
        let matches = match_path_impl(&x, MatcherSettings::default(), "/anything/thing/thing")
            .expect("should match")
            .1;
        assert_eq!(matches["cap"], "anything".to_string())
    }

    #[test]
    fn match_path_5() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{cap}/thing", true)
            .expect("Should parse");
        let matches = match_path_impl(&x, MatcherSettings::default(), "/anything/thing/thing")
            .expect("should match")
            .1;
        assert_eq!(matches["cap"], "anything".to_string())
    }

    #[test]
    fn match_fragment() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("#test", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
    }

    #[test]
    fn match_fragment_after_path() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/#test", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path/#test").expect("should match");
    }

    #[test]
    fn match_fragment_after_path_no_slash() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path#test", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path#test").expect("should match");
    }

    #[test]
    fn match_fragment_after_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens(
            "/a/path?query=thing#test",
            true,
        )
        .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path?query=thing#test")
            .expect("should match");
    }

    #[test]
    fn match_fragment_after_query_capture() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens(
            "/a/path?query={capture}#test",
            true,
        )
        .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path?query=thing#test")
            .expect("should match");
    }

    #[test]
    fn match_fragment_optional() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("(#test)", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "").expect("should match");
    }
    #[test]
    fn match_fragment_pound_optional() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("#(test)", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "#").expect("should match");
    }

    #[test]
    fn match_fragment_optional_with_inner_optional_item() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("(#(test))", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "#").expect("should match");
    }

    #[test]
    fn capture_as_only_token() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("{any}", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "literally_anything")
            .expect("should match");
    }

    #[test]
    fn optional_path_first() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("(/thing)", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/thing").expect("should match");
    }

    #[test]
    fn optional_path_after_item() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/first(/second)", true)
            .expect("Should parse");
        assert_eq!(
            x,
            vec![
                MatcherToken::Exact("/first".to_string()),
                MatcherToken::Optional(vec![MatcherToken::Exact("/second".to_string())]),
                MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())])
            ]
        );
        match_path_impl(&x, MatcherSettings::default(), "/first").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/first/second").expect("should match");
    }

    #[test]
    fn optional_path_any() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/first(/{})", true)
            .expect("Should parse");
        let expected = vec![
            MatcherToken::Exact("/first".to_string()),
            MatcherToken::Optional(vec![
                MatcherToken::Exact("/".to_string()),
                MatcherToken::Capture(CaptureVariant::Unnamed),
            ]),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        assert_eq!(x, expected);
        match_path_impl(&x, MatcherSettings::default(), "/first").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/first/second").expect("should match");
    }

    #[test]
    fn optional_path_capture_all() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*}(/stuff)", true)
            .expect("Should parse");
        let expected = vec![
            MatcherToken::Exact("/".to_string()),
            MatcherToken::Capture(CaptureVariant::ManyUnnamed),
            MatcherToken::Optional(vec![MatcherToken::Exact("/stuff".to_string())]),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        assert_eq!(x, expected);
        match_path_impl(&x, MatcherSettings::default(), "/some/garbage").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/some/garbage/stuff")
            .expect("should match");
    }

    #[test]
    fn case_insensitive() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/hello", true)
            .expect("Should parse");
        let settings = MatcherSettings {
            case_insensitive: true,
            ..Default::default()
        };
        match_path_impl(&x, settings, "/HeLLo").expect("should match");
    }
}
