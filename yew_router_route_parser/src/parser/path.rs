use nom::IResult;
use nom::branch::alt;
use nom::sequence::tuple;
use crate::parser::Token;
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::bytes::complete::tag;
use crate::parser::core::{capture, match_specific_token};

/// Handles either a leading '/' or  a '/thing'
pub fn path_parser(i: &str) -> IResult<&str, Vec<Token>> {
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


fn separator_token(i: &str) -> IResult<&str, Token> {
    map(
        tag("/"),
        |_| Token::Separator
    )(i)
}


pub fn section_matchers(i: &str) -> IResult<&str, Vec<Token>> {

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

#[cfg(test)]
mod test {
    use super::*;
    use nom::combinator::all_consuming;

    #[test]
    fn path_must_start_with_separator() {
        path_parser("hello").expect_err("Should reject at absence of /");
        let parsed = super::super::parse("/hello").expect("should parse");
        assert_eq!(parsed.1, vec![Token::Separator, Token::Match("hello".to_string())])
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
        all_consuming(path_parser)("/path{}{}").expect_err("Should not validate");
    }


    #[test]
    fn section_matchers_falis_to_match() {
        section_matchers("{aoeu").expect_err("Should not complete");
    }

}
