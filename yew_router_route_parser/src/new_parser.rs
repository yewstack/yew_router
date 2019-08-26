use nom::IResult;
use nom::branch::alt;
use nom::sequence::{delimited, separated_pair, tuple, preceded};
use nom::bytes::complete::{tag, is_not, take};
use nom::combinator::{map, opt, all_consuming, peek};
use nom::error::{ParseError, ErrorKind};
use nom::multi::{many1, many0};
use nom::character::is_digit;
use nom::character::complete::{digit1};
use std::hint::unreachable_unchecked;


#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Separator,
    Match(String), // Any string
    Capture(CaptureVariant), // {_}
    QueryBegin, // ?
    QuerySeparator, // &
    QueryCapture{ident: String, capture_or_match: CaptureOrMatch}, // x=y
    FragmentBegin, // #
    Optional(Vec<Token>)
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

pub fn parse(i: &str) -> IResult<&str, Vec<Token>> {
    map(
        all_consuming(tuple(
            (
                opt(path_parser),
                opt(query_parser),
                opt(framgent_parser)
            )
        )),
        |(path, query, fragment): (Option<Vec<Token>>, Option<Vec<Token>>, Option<Vec<Token>>)| {
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
    )(i)
}


/// Handles either a leading '/' or  a '/thing'
fn path_parser(i: &str) -> IResult<&str, Vec<Token>> {
    fn inner_path_parser(i: &str) -> IResult<&str, (Token, Vec<Token>)> {
        tuple(
            (
                separator_token,
                section_matchers
            )
        )(i)
    }

    let many_inner_paths = map(
        many1(inner_path_parser),
        |tokens: Vec<(Token, Vec<Token>)>| {
            let new_capacity = tokens.capacity() * 2;
            tokens.into_iter().fold(Vec::with_capacity(new_capacity), |mut accumulator, mut element| {
                accumulator.push(element.0);
                accumulator.append(&mut element.1);
                accumulator
            })
        });


    // accept any number of /thing or just '/
    alt(
        (
            map(
                tuple((many_inner_paths, opt(separator_token))),
                |(mut paths, ending_separator)| {
                    if let Some(end_sep) = ending_separator {
                        paths.push(end_sep)
                    }
                    paths
                }
            ),
            map(separator_token,
                |x| vec![x])
        )
    )(i)
}


/// Matches:
/// * "?query=item"
/// * "?query=item&query2=item"
/// * "?query=item&query2=item&query3=item"
/// * "?query={capture}"
/// * "?query={capture}&query2=item"
/// * etc...
fn query_parser(i: &str) -> IResult<&str, Vec<Token>> {
    fn begin_query_parser(i: &str) -> IResult<&str, (Token, Token)> {
        tuple(
            (
                query_begin_token,
                query
            )
        )(i)
    }

    fn rest_query_parser(i: &str) -> IResult<&str, Vec<Token>> {
        map(
            many0(tuple(
                (
                    query_separator_token,
                    query
                )
            )),
            |tokens: Vec<(Token, Token)>| {
                let new_capacity = tokens.capacity() * 2;
                tokens.into_iter().fold(Vec::with_capacity(new_capacity), |mut accumulator, element| {
                    accumulator.push(element.0);
                    accumulator.push(element.1);
                    accumulator
                })

            }
        )(i)
    }

    map(
        tuple(
            (
                begin_query_parser,
                rest_query_parser
            )
        ),
        |(first, mut rest)| {
            let mut tokens = vec![first.0, first.1];
            tokens.append(&mut rest);
            tokens
        }
    )(i)
}

fn framgent_parser(i: &str) -> IResult<&str, Vec<Token>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, mut section) = section_matchers(i)?;
    let mut v = vec![begin];
    v.append(&mut section);
    Ok((i, v))
}


/// Captures a string up to the point where a character not possible to be present in Rust's identifier is encountered.
/// It prevents the first character from being a digit.
pub fn valid_ident_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " -*/+#?&^@~`;,.|\\{}[]=\t\n";
    let (i, next) = peek(take(1usize))(i)?; // Look at the first character
    if is_digit(next.bytes().next().unwrap()) {
        return Err(nom::Err::Error((i, ErrorKind::Digit))) // Digits not allowed
    } else {
        is_not(INVALID_CHARACTERS)(i)
    }
}

fn match_specific_token(i: &str) -> IResult<&str, Token> {
    map(
        valid_ident_characters,
        |ident| Token::Match(ident.to_string())
    )(i)
}

fn capture(i: &str) -> IResult<&str, Token> {
    let capture_variants = alt(
        (
            map(peek(tag("}")), |_| CaptureVariant::Unnamed), // just empty {}
            map(preceded(tag("*:"), valid_ident_characters), |s| CaptureVariant::ManyNamed(s.to_string())),
            map(tag("*"), |_| CaptureVariant::ManyUnnamed),
            map(valid_ident_characters, |s| CaptureVariant::Named(s.to_string())),
            map(separated_pair(digit1, tag(":"), valid_ident_characters), |(n, s)| CaptureVariant::NumberedNamed {sections: n.parse().expect("Should parse digits"), name: s.to_string()}),
            map(digit1, |num: &str| CaptureVariant::NumberedUnnamed {sections: num.parse().expect("should parse digits" )})
        )
    );

    map(
        delimited(tag("{"), capture_variants, tag("}")),
        Token::Capture
    )(i)
}

/// matches "item=item" and "item={capture}"
fn query(i: &str) -> IResult<&str, Token> {
    map(
        separated_pair(valid_ident_characters, tag("=",), capture_or_match),
        |(ident, value)| Token::QueryCapture {ident: ident.to_string(), capture_or_match: value }
    )(i)
}

/// Matches either "item" or "{capture}"
/// It returns a subset enum of Token.
fn capture_or_match(i: &str) -> IResult<&str, CaptureOrMatch> {
    let (i, token) = alt((capture, match_specific_token))(i)?;
    let token = match token {
        Token::Capture(variant) => CaptureOrMatch::Capture(variant),
        Token::Match(m) => CaptureOrMatch::Match(m),
        _ => unreachable!("Only should handle captures and matches")
    };
    Ok((i, token))
}

fn separator_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("/"),
        |_| Token::Separator
    )(i)
}

fn query_begin_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("?"),
        |_| Token::QueryBegin
    )(i)
}
fn query_separator_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("&"),
        |_| Token::QuerySeparator
    )(i)
}
fn begin_fragment_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("#"),
        |_| Token::FragmentBegin
    )(i)
}

fn section_matchers(i: &str) -> IResult<&str, Vec<Token>> {

    let (i, token): (&str, Token) = alt((match_specific_token, capture))(i)?;
    let tokens = vec![token];

    /// You can't have two matching sections in a row, because there is nothing to indicate when
    /// one ends and the other begins.
    /// This function collects possible section matchers and prevents them auto-glob matchers
    /// from residing next to each other.
    fn match_next_section_matchers(i: &str, mut tokens: Vec<Token>) -> IResult<&str, Vec<Token>> {
        let token = tokens.last().expect("Must be at least one token.");
        match token {
            Token::Match(_) => {
                let (i, t) = opt( capture)(i)?;
                if let Some(new_t) = t {
                    tokens.push(new_t);
                    match_next_section_matchers(i, tokens)
                } else {
                    Ok((i,tokens))
                }
            },
            Token::Capture(_) => {
                let (i, t) = opt(match_specific_token)(i)?;
                if let Some(new_t) = t {
                    tokens.push(new_t);
                    match_next_section_matchers(i, tokens)
                } else {
                    Ok((i,tokens))
                }


            },
            _ => unreachable!()
        }
    }

    match_next_section_matchers(i, tokens)
}



fn ret_vec<F: Fn(&str) -> IResult<&str, Token>>(f: F) -> impl Fn(&str) -> IResult<&str, Vec<Token>> {
    move |i: &str | {
        (f)(i).map(|(i, t)| (i, vec![t]))
    }
}

fn optional_section<F: Fn(&str) -> IResult<&str, Vec<Token>>>(f: F) -> impl Fn(&str) -> IResult<&str, Token> {
    move |i: &str| -> IResult<&str, Token> {
        let f = &f;
        delimited(tag("("), f, tag(")"))(i)
            .map(|(i, t)| (i, Token::Optional(t)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_named_test() {
        let cap = capture("{hellothere}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::Named("hellothere".to_string()))));
    }

    #[test]
    fn capture_many_unnamed_test() {
        let cap = capture("{*}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::ManyUnnamed)));
    }

    #[test]
    fn capture_unnamed_test() {
        let cap = capture("{}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::Unnamed)));
    }

    #[test]
    fn capture_numbered_unnamed_test() {
        let cap = capture("{5}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::NumberedUnnamed {sections: 5})));
    }

    #[test]
    fn capture_numbered_named_test() {
        let cap = capture("{5:name}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::NumberedNamed{sections: 5, name: "name".to_string()})));
    }


    #[test]
    fn capture_many_named() {
        let cap = capture("{*:name}").unwrap();
        assert_eq!(cap, ("", Token::Capture (CaptureVariant::ManyNamed("name".to_string()))));
    }


    #[test]
    fn rejects_invalid_ident() {
        valid_ident_characters("+-Hello").expect_err("Should reject at +");
    }

    #[test]
    fn accepts_valid_ident() {
        valid_ident_characters("Hello").expect("Should accept");
    }

//    #[test]
//    fn match_any() {
//        match_any_token("*").expect("Should match");
//    }

    #[test]
    fn path_must_start_with_separator() {
        path_parser("hello").expect_err("Should reject at absence of /");
        let parsed = parse("/hello").expect("should parse");
        assert_eq!(parsed.1, vec![Token::Separator, Token::Match("hello".to_string())])
    }

    #[test]
    fn parse_can_handle_multiple_literals() {
        let parsed = parse("/hello/there").expect("should parse");
        assert_eq!(parsed.1, vec![Token::Separator,
                                  Token::Match("hello".to_string()),
                                  Token::Separator,
                                  Token::Match("there".to_string())]
        );
    }



    #[test]
    fn parse_can_handle_trailing_path_separator() {
        let parsed = parse("/hello/").expect("should parse");
        assert_eq!(parsed.1, vec![Token::Separator,
                                  Token::Match("hello".to_string()),
                                  Token::Separator]
        );
    }

    #[test]
    fn parse_can_capture_section() {
        let parsed = parse("/hello/{there}").expect("should parse");
        assert_eq!(parsed.1, vec![
            Token::Separator,
            Token::Match("hello".to_string()),
            Token::Separator,
            Token::Capture(CaptureVariant::Named("there".to_string())),
        ]
        )
    }

    #[test]
    fn parse_can_handle_multiple_matches_per_section() {
        let parsed = parse("/hello/{there}general{}").expect("should parse");
        assert_eq!(parsed.1, vec![
            Token::Separator,
            Token::Match("hello".to_string()),
            Token::Separator,
            Token::Capture(CaptureVariant::Named("there".to_string())),
            Token::Match("general".to_string()),
            Token::Capture(CaptureVariant::Unnamed)
        ]
        )
    }
    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_0() {
        all_consuming(path_parser)("/path*{match}").expect_err("Should not validate");
    }
    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_1() {
        all_consuming(path_parser)("/path{match1}{match2}").expect_err("Should not validate");
    }
    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_2() {
        all_consuming(path_parser)("/path**").expect_err("Should not validate");
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
        parse("/path**").expect_err("Should not validate");
    }

    #[test]
    fn parse_consumes_all_input() {
        parse("/hello/{").expect_err("Should not complete");
    }

    #[test]
    fn capture_consumes() {
        capture("{aoeu").expect_err("Should not complete");
    }

    #[test]
    fn section_matchers_falis_to_match() {
        section_matchers("{aoeu").expect_err("Should not complete");
    }

}
