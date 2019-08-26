use yew_router_route_parser::{CaptureVariants, OptimizedToken};
use nom::IResult;
use crate::Matches;
use nom::bytes::complete::{tag, take_until, is_not};
use nom::sequence::{terminated, preceded};
use nom::combinator::{peek, opt};
use log::{trace, debug};
use std::iter::Peekable;
use std::slice::Iter;
use nom::error::ErrorKind;

pub(super) fn match_paths<'a, 'b>(tokens: &'b Vec<OptimizedToken>, mut i: &'a str) -> IResult<&'a str, Matches<'b>> {
    debug!("Attempting to match path: {:?} using: {:?}", i, tokens);

    let mut iter = tokens
        .iter()
        .peekable();

    let mut matches: Matches = Matches::new();

    while let Some(token) = iter.next() {
        i = match token {
            OptimizedToken::Match(literal) => {
                trace!("Matching literal: {}", literal);
                tag(literal.as_str())(i)?.0
            },
            OptimizedToken::Optional(inner_tokens) => {
                match opt(|i|{
                    match_paths(inner_tokens, i)
                })(i) {
                    Ok((ii, inner_matches)) => {
                        //TODO needs some tests to verify if the following is right (handling of i)
                        if let Some(inner_matches) = inner_matches {
                            matches.extend(inner_matches);
                        }
                        ii
                    }
                    _ => i // Do nothing if this fails
                }
            }
            OptimizedToken::Capture (variant) => {
                match variant {
                    CaptureVariants::Unnamed => capture_unnamed(i, &mut iter)?,
                    CaptureVariants::ManyUnnamed => capture_many_unnamed(i, &mut iter)?,
                    CaptureVariants::NumberedUnnamed { sections } => capture_numbered_unnamed(i, &mut iter, *sections)?,
                    CaptureVariants::Named(name) => capture_named(i, &mut iter, &name, &mut matches )?,
                    CaptureVariants::ManyNamed(name) => capture_many_named(i, &mut iter, &name, &mut matches )?,
                    CaptureVariants::NumberedNamed { sections, name } => capture_numbered_named(i, &mut iter, &name, *sections, &mut matches)?
                }
            },
        };
    }
    debug!("Path Matched");

    Ok((i, matches))
}


/// Captures a section and doesn't add it to the matches.
///
/// It will capture characters until a separator or other invalid character is encountered
/// and the next string of characters is confirmed to be the next literal.
fn capture_unnamed<'a>(i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Unnamed");
    let ii = if let Some(peaked_next_token) = iter.peek() {
        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
        terminated(valid_capture_characters, peek(tag(delimiter)))(i)?.0
    } else {
        if i.len() == 0 {
            i // Match even if nothing is left
        } else if i == "/" {
            "" // Trailing '/' is allowed
        } else {
            valid_capture_characters(i)?.0
        }
    };
    Ok(ii)
}

fn capture_many_unnamed<'a>(i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    trace!("Matching ManyUnnamed");
    let ii = if let Some(peaked_next_token) = iter.peek() {
        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
        take_until(delimiter)(i)?.0
    } else {
        if i.len() == 0 {
            i // Match even if nothing is left
        } else {
            valid_many_capture_characters(i)?.0
        }
    };
    Ok(ii)
}

fn capture_numbered_unnamed<'a>(mut i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>, mut sections: usize) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedUnnamed ({})", sections);
    if let Some(peaked_next_token) = iter.peek() {
        while sections > 0 {
            if sections > 1 {
                i = terminated(valid_capture_characters, tag("/"))(i)?.0;
            } else {
                // Don't consume the next character on the last section
                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                i = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?.0;
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

fn capture_named<'a, 'b>(i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>, capture_key: &'b str, matches: &mut Matches<'b>) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching Named ({})", capture_key);
    if let Some(peaked_next_token) = iter.peek() {
        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
        let (ii, captured) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
        matches.insert(capture_key, captured.to_string());
        Ok(ii)
    } else {
        let (ii, captured) = valid_capture_characters(i)?;
        matches.insert(capture_key, captured.to_string());
        Ok(ii)
    }
}


fn capture_many_named<'a, 'b>(i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>, capture_key: &'b str, matches: &mut Matches<'b>) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedUnnamed ({})", capture_key);
    if let Some(peaked_next_token) = iter.peek() {
        let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("Should be in sequence");
        let (ii, c) = take_until(delimiter)(i)?;
        matches.insert(&capture_key, c.to_string());
        Ok(ii)
    } else {
        if i.len() == 0 {
            matches.insert(&capture_key, "".to_string()); // Is this a thing I want?
            Ok(i) // Match even if nothing is left
        } else {
            let (ii, c) = valid_many_capture_characters(i)?;
            matches.insert(&capture_key, c.to_string());
            Ok(ii)
        }
    }
}

fn capture_numbered_named<'a, 'b>(mut i: &'a str, iter: &mut Peekable<Iter<OptimizedToken>>, name: &'b str, mut sections: usize, matches: &mut Matches<'b>) -> Result<&'a str, nom::Err<(&'a str, ErrorKind)>> {
    log::trace!("Matching NumberedNamed ({}, {})", sections, name);
    let mut captured = "".to_string();
    if let Some(peaked_next_token) = iter.peek() {
        while sections > 0 {
            if sections > 1 {
                let (ii, c) = terminated(valid_capture_characters, tag("/"))(i)?;
                i = ii;
                captured += c;
                captured += "/";
            } else {
                // Don't consume the next character on the last section
                let delimiter = peaked_next_token.lookup_next_concrete_sequence().expect("should be in sequence");
                let (ii, c) = terminated(valid_capture_characters, peek(tag(delimiter)))(i)?;
                i = ii;
                captured += c;
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
    const INVALID_CHARACTERS: &str = " #&?{}=";
    is_not(INVALID_CHARACTERS)(i)
}

fn valid_capture_characters_in_query(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " *#&?|{}=";
    is_not(INVALID_CHARACTERS)(i)
}


#[cfg(test)]
mod integration_test {
    use super::*;



    #[test]
    fn match_query_after_path() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path?hello=there").expect("Should parse");
        match_paths(&x, "/a/path?hello=there").expect("should match");
    }

    #[test]
    fn match_query_after_path_trailing_slash() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/?hello=there").expect("Should parse");
        match_paths(&x, "/a/path/?hello=there").expect("should match");
    }

// TODO this should be able to be less strict. A trailing slash before a # or ? should be ignored

//    #[test]
//    fn match_query_after_path_slash_ignored() {
//        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/?hello=there").expect("Should parse");
//        match_paths(&x, "/a/path?hello=there").expect("should match");
//    }

    #[test]
    fn match_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?hello=there").expect("Should parse");
        match_paths(&x, "?hello=there").expect("should match");
    }

    #[test]
    fn named_capture_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("?hello={there}").expect("Should parse");
        let (_, matches) = match_paths(&x, "?hello=there").expect("should match");
        assert_eq!(matches["there"], "there".to_string())
    }




    #[test]
    fn match_fragment() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("#test").expect("Should parse");
        match_paths(&x, "#test").expect("should match");
    }


    #[test]
    fn match_fragment_after_path() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path/#test").expect("Should parse");
        match_paths(&x, "/a/path/#test").expect("should match");
    }

    #[test]
    fn match_fragment_after_path_no_slash() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path#test").expect("Should parse");
        match_paths(&x, "/a/path#test").expect("should match");
    }

    #[test]
    fn match_fragment_after_query() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path?query=thing#test").expect("Should parse");
        match_paths(&x, "/a/path?query=thing#test").expect("should match");
    }


    #[test]
    fn match_fragment_after_query_capture() {
        let x = yew_router_route_parser::parse_str_and_optimize_tokens("/a/path?query={capture}#test").expect("Should parse");
        match_paths(&x, "/a/path?query=thing#test").expect("should match");
    }
}