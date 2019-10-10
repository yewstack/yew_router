use crate::matcher::route_matcher::util::tag_possibly_case_sensitive;
use crate::matcher::route_matcher::MatcherSettings;
use crate::matcher::Captures;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::combinator::{map, opt, verify};
use nom::error::ErrorKind;
use nom::sequence::terminated;
use nom::IResult;
use std::iter::Peekable;
use std::slice::Iter;
use yew_router_route_parser::parser::util::consume_until;
use yew_router_route_parser::{CaptureVariant, MatcherToken};

/// Allows abstracting over capturing into a HashMap (Captures) or a Vec.
pub trait CaptureCollection<'a> {
    fn new2() -> Self;
    fn insert2(&mut self, key: &'a str, value: String);
    fn extend2(&mut self, other: Self);
}

impl<'a> CaptureCollection<'a> for Captures<'a> {
    fn new2() -> Self {
        Captures::new()
    }

    fn insert2(&mut self, key: &'a str, value: String) {
        self.insert(key, value);
    }

    fn extend2(&mut self, other: Self) {
        self.extend(other)
    }
}

impl<'a> CaptureCollection<'a> for Vec<(&'a str, String)> {
    fn new2() -> Self {
        Vec::new()
    }

    fn insert2(&mut self, key: &'a str, value: String) {
        self.push((key, value))
    }

    fn extend2(&mut self, other: Self) {
        self.extend(other)
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn match_path<'a, 'b: 'a>(
    tokens: &'b [MatcherToken],
    settings: &'b MatcherSettings,
) -> impl Fn(&'a str) -> IResult<&'a str, Captures<'b>> {
    move |i: &str| match_path_impl(tokens, *settings, i)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn match_path_list<'a, 'b: 'a>(
    tokens: &'b [MatcherToken],
    settings: &'b MatcherSettings,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<(&'b str, String)>> {
    move |i: &str| match_path_impl(tokens, *settings, i)
}

fn match_path_impl<'a, 'b: 'a, CAP: CaptureCollection<'b>>(
    tokens: &'b [MatcherToken],
    settings: MatcherSettings,
    mut i: &'a str,
) -> IResult<&'a str, CAP> {
    trace!("Attempting to match path: {:?} using: {:?}", i, tokens);

    let mut iter = tokens.iter().peekable();

    let mut captures: CAP = CAP::new2();

    while let Some(token) = iter.next() {
        i = match token {
            MatcherToken::Exact(literal) => {
                trace!("Matching '{}' against literal: '{}'", i, literal);
                tag_possibly_case_sensitive(literal.as_str(), !settings.case_insensitive)(i)?.0
            }
            MatcherToken::Optional(inner_tokens) => {
                match opt(|i| match_path_impl(&inner_tokens, settings, i))(i) {
                    Ok((ii, inner_captures)) => {
                        if let Some(inner_captures) = inner_captures {
                            captures.extend2(inner_captures);
                        }
                        ii
                    }
                    _ => i, // Do nothing if this fails
                }
            }
            MatcherToken::Capture(capture) => match &capture.capture_variant {
                CaptureVariant::Unnamed => {
                    capture_unnamed(i, &mut iter, &capture.allowed_captures)?
                }
                CaptureVariant::ManyUnnamed => {
                    capture_many_unnamed(i, &mut iter, &capture.allowed_captures)?
                }
                CaptureVariant::NumberedUnnamed { sections } => capture_numbered_named::<CAP>(
                    i,
                    &mut iter,
                    None,
                    *sections,
                    &capture.allowed_captures,
                )?,
                CaptureVariant::Named(name) => capture_named(
                    i,
                    &mut iter,
                    &name,
                    &mut captures,
                    &capture.allowed_captures,
                )?,
                CaptureVariant::ManyNamed(name) => capture_many_named(
                    i,
                    &mut iter,
                    &name,
                    &mut captures,
                    &capture.allowed_captures,
                )?,
                CaptureVariant::NumberedNamed { sections, name } => capture_numbered_named(
                    i,
                    &mut iter,
                    Some((&name, &mut captures)),
                    *sections,
                    &capture.allowed_captures,
                )?,
            },
        };
    }
    debug!("Path Matched");

    Ok((i, captures))
}

// TODO This section of code is kind of a mess. It needs a pretty through rework.

/// Captures a section and doesn't add it to the matches.
///
/// It will capture characters until a separator or other invalid character is encountered
/// and the next string of characters is confirmed to be the next literal.
pub fn capture_unnamed<'a>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    allowed_captures: &Option<Vec<String>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Unnamed");
    let ii = if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        let matcher = alt((
            consume_until(alt((tag("/"), tag("?"), tag("#")))),
            consume_until(delimiter),
        ));
        optionally_check_if_parsed_is_allowed_capture(matcher, allowed_captures)(i)?.0
    } else if i.is_empty() {
        i // Match even if nothing is left
    } else {
        let valid_capture_characters = map(valid_capture_characters, String::from);
        optionally_check_if_parsed_is_allowed_capture(valid_capture_characters, allowed_captures)(
            i,
        )?
        .0
    };
    Ok(ii)
}

fn capture_many_unnamed<'a>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    allowed_captures: &Option<Vec<String>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    trace!("Matching ManyUnnamed");
    let ii = if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        optionally_check_if_parsed_is_allowed_capture(consume_until(delimiter), allowed_captures)(
            i,
        )?
        .0
    } else if i.is_empty() {
        i // Match even if nothing is left
    } else {
        let valid_many_capture_characters = map(valid_many_capture_characters, String::from);
        optionally_check_if_parsed_is_allowed_capture(
            valid_many_capture_characters,
            allowed_captures,
        )(i)?
        .0
    };
    Ok(ii)
}

fn capture_named<'a, 'b: 'a, CAP: CaptureCollection<'b>>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    capture_key: &'b str,
    matches: &mut CAP,
    allowed_captures: &Option<Vec<String>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Named ({})", capture_key);
    if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        let (ii, captured) = optionally_check_if_parsed_is_allowed_capture(
            consume_until(delimiter),
            allowed_captures,
        )(i)?;
        matches.insert2(capture_key, captured);
        Ok(ii)
    } else {
        let (ii, captured) = optionally_check_if_parsed_is_allowed_capture(
            map(valid_capture_characters, String::from),
            allowed_captures,
        )(i)?;
        matches.insert2(capture_key, captured.to_string());
        Ok(ii)
    }
}

fn capture_many_named<'a, 'b, CAP: CaptureCollection<'b>>(
    i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    capture_key: &'b str,
    matches: &mut CAP,
    allowed_captures: &Option<Vec<String>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedUnnamed ({})", capture_key);
    if let Some(_peaked_next_token) = iter.peek() {
        let delimiter = yew_router_route_parser::next_delimiters(iter.clone());
        let (ii, captured) = optionally_check_if_parsed_is_allowed_capture(
            consume_until(delimiter),
            allowed_captures,
        )(i)?;
        matches.insert2(&capture_key, captured);
        Ok(ii)
    } else if i.is_empty() {
        matches.insert2(&capture_key, "".to_string()); // TODO Is this a thing I want?
        Ok(i) // Match even if nothing is left
    } else {
        let (ii, c) = optionally_check_if_parsed_is_allowed_capture(
            map(valid_many_capture_characters, String::from),
            allowed_captures,
        )(i)?;
        matches.insert2(&capture_key, c.to_string());
        Ok(ii)
    }
}

fn capture_numbered_named<'a, 'b, CAP: CaptureCollection<'b>>(
    mut i: &'a str,
    iter: &mut Peekable<Iter<MatcherToken>>,
    name_and_captures: Option<(&'b str, &mut CAP)>,
    mut sections: usize,
    allowed_captures: &Option<Vec<String>>,
) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedNamed ({})", sections);
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

    if let Some(allowed_captures) = allowed_captures {
        if allowed_captures.iter().any(|x| x == &captured) {
            if let Some((name, captures)) = name_and_captures {
                captures.insert2(&name, captured);
            }
            Ok(i)
        } else {
            Err(nom::Err::Error((i, ErrorKind::Verify)))
        }
    } else {
        if let Some((name, captures)) = name_and_captures {
            captures.insert2(&name, captured);
        }
        Ok(i)
    }
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

/// If the allowed matches is Some, then check to see if any of the elements in it are what was captured.
/// It will fail if the captured value was not in the provided list.
///
/// If none, it will allow the capture as intended.
fn optionally_check_if_parsed_is_allowed_capture<'a, F: 'a>(
    f: F,
    allowed_captures: &Option<Vec<String>>,
) -> impl Fn(&'a str) -> IResult<&'a str, String, (&'a str, ErrorKind)>
//Result<&'a str, nom::Err<(&'a str, ErrorKind)>>
where
    F: Fn(&'a str) -> IResult<&'a str, String, (&'a str, ErrorKind)>,
{
    let am = allowed_captures.clone(); // I hate how this has to clone in order to work.
    map(
        verify(f, move |s: &String| {
            if let Some(am) = &am {
                am.iter().map(String::as_str).any(|x| x == s)
            } else {
                true
            }
        }),
        |o| o.to_string(),
    )
}

//fn valid_capture_characters_in_query(i: &str) -> IResult<&str, &str> {
//    const INVALID_CHARACTERS: &str = " *#&?|{}=";
//    is_not(INVALID_CHARACTERS)(i)
//}

#[cfg(test)]
mod integration_test {
    use super::*;

    use yew_router_route_parser;
    use yew_router_route_parser::Capture;
    //    use nom::combinator::all_consuming;

    #[test]
    fn match_query_after_path() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path?lorem=ipsum", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path?lorem=ipsum")
            .expect("should match");
    }

    #[test]
    fn match_query_after_path_trailing_slash() {
        let x =
            yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/?lorem=ipsum", true)
                .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/a/path/?lorem=ipsum")
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
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?lorem=ipsum", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "?lorem=ipsum").expect("should match");
    }

    #[test]
    fn named_capture_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?lorem={ipsum}", true)
            .expect("Should parse");
        let (_, matches) =
            match_path_impl(&x, MatcherSettings::default(), "?lorem=ipsum").expect("should match");
        assert_eq!(matches["ipsum"], "ipsum".to_string())
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
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("[#test]", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "").expect("should match");
    }
    #[test]
    fn match_fragment_pound_optional() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("#[test]", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "#test").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "#").expect("should match");
    }

    #[test]
    fn match_fragment_optional_with_inner_optional_item() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("[#[test]]", true)
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
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("[/thing]", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/thing").expect("should match");
    }

    #[test]
    fn optional_path_after_item() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/first[/second]", true)
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
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/first[/{}]", true)
            .expect("Should parse");
        let expected = vec![
            MatcherToken::Exact("/first".to_string()),
            MatcherToken::Optional(vec![
                MatcherToken::Exact("/".to_string()),
                MatcherToken::Capture(Capture::from(CaptureVariant::Unnamed)),
            ]),
            MatcherToken::Optional(vec![MatcherToken::Exact("/".to_string())]),
        ];
        assert_eq!(x, expected);
        match_path_impl(&x, MatcherSettings::default(), "/first").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/first/second").expect("should match");
    }

    #[test]
    fn optional_path_capture_all() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{*}[/stuff]", true)
            .expect("Should parse");
        let expected = vec![
            MatcherToken::Exact("/".to_string()),
            MatcherToken::Capture(Capture::from(CaptureVariant::ManyUnnamed)),
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

    #[test]
    fn match_limited_1() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/{(cap)}/thing", true)
            .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/cap/thing").expect("should match");
    }

    #[test]
    fn match_limited_2() {
        let x =
            yew_router_route_parser::parse_str_and_optimize_tokens("/{(other|cap)}/thing", true)
                .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/cap/thing").expect("should match");
        match_path_impl(&x, MatcherSettings::default(), "/other/thing").expect("should match");
    }

    #[test]
    fn match_limited_3() {
        let x =
            yew_router_route_parser::parse_str_and_optimize_tokens("/{key(other|cap)}/thing", true)
                .expect("Should parse");
        let captures = match_path_impl(&x, MatcherSettings::default(), "/cap/thing")
            .expect("should match")
            .1;
        assert_eq!(captures["key"], "cap".to_string())
    }

    #[test]
    fn match_limited_4() {
        let x =
            yew_router_route_parser::parse_str_and_optimize_tokens("/{key(other|cap)}/thing", true)
                .expect("Should parse");
        match_path_impl(&x, MatcherSettings::default(), "/snails/thing")
            .expect_err("should not match");
    }
}
