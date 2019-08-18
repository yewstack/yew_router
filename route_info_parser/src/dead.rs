use std::collections::HashMap;
use nom::IResult;
use nom::bytes::complete::{take_until, tag};
use nom::sequence::preceded;
use crate::parser::Token;

pub fn create_path_matcher(mut i: &str, tokens: Vec<Token>) -> IResult<&str, HashMap<String, String>> {
    let mut iter = tokens
        .into_iter()
        .peekable();


    let mut dictionary = HashMap::new();

    while let Some(token) = iter.next() {
        match dbg!(token) {
            Token::Separator => {
                let (ii, _) = tag("/")(i)?;
                i = ii;
            },
            Token::Match(literal) => {
                let (ii, _) = tag(literal.as_str())(i)?;
                i = ii;
            },
            Token::MatchAny => {
                if let Some(peaked_next_token) = iter.peek() {
                    let delimiter = delimiter_lookup(peaked_next_token);
                    let (ii, _captured) =  take_until(delimiter)(i)?;
                    i = ii;
                } else {
                    let (ii, _captured) = crate::parser::valid_ident_characters(i)?;
                    i = ii;
                }
            },
            Token::Capture { ident: capture_key } => {
                if let Some(peaked_next_token) = iter.peek() {
                    let delimiter = delimiter_lookup(peaked_next_token);
                    let (ii, captured) =  take_until(delimiter)(i)?;
                    i = ii;
                    dictionary.insert(capture_key, captured.to_string());
                } else {
                    let (ii, captured) = crate::parser::valid_ident_characters(i)?;
                    i = ii;
                    dictionary.insert(capture_key, captured.to_string());
                }
            },
            Token::QueryBegin => {
                let (ii, _) = tag("?")(i)?;
                i = ii;
            },
            Token::QuerySeparator => {
                let (ii, _) = tag("&")(i)?;
                i = ii;
            },
            Token::QueryCapture { ident,  value: capture_key} => {
                if let Some(peaked_next_token) = iter.peek() {
                    let delimiter = delimiter_lookup(peaked_next_token);
                    let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), take_until(delimiter))(i)?; // TODO this should also probably prevent
                    i = ii;
                    dictionary.insert(capture_key, captured.to_string());
                } else {
                    let (ii, captured) = preceded(tag(format!("{}=", ident).as_str()), crate::parser::valid_ident_characters)(i)?; // TODO, should allow '/' characters in query value?
                    i = ii;
                    dictionary.insert(capture_key, captured.to_string());
                }
            },
            Token::FragmentBegin => {
                let (ii, _) = tag("#")(i)?;
                i = ii;
            },
        };
    }
    Ok((i, dictionary))
}


pub fn delimiter_lookup(token: &Token) -> &str {
    match token {
        Token::Separator => "/",
        Token::Match(literal) => &literal,
        Token::MatchAny => unreachable!(),
        Token::Capture { ident: _ } => unreachable!(),
        Token::QueryBegin => "?",
        Token::QuerySeparator => "&",
        Token::QueryCapture { ident: _, value: _ } => { unimplemented!()} // TODO implement me, or maybe mark unreachable?
        Token::FragmentBegin => "#",
    }
}
