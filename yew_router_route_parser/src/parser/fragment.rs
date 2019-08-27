use nom::IResult;
use crate::parser::RouteParserToken;
use crate::parser::util::{optional_matches, optional_matches_v};
use crate::parser::path::section_matchers; // TODO possibly duplicate this function (loosen its requirements for this module eg. allow '/' characters.)
use nom::branch::alt;
use nom::combinator::map;
use nom::bytes::complete::tag;


fn begin_fragment_token(i: &str) -> IResult<&str, RouteParserToken> {
    map(
        tag("#"),
        |_| RouteParserToken::FragmentBegin
    )(i)
}

/// #item
fn simple_fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, mut section) = section_matchers(i)?;
    let mut v = vec![begin];
    v.append(&mut section);
    Ok((i, v))
}

/// #(item)
fn fragment_parser_with_optional_item(i: &str) -> IResult<&str, Vec<RouteParserToken>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, optional) = optional_matches(section_matchers)(i)?;
    let v = vec![begin, optional];
    Ok((i, v))
}

/// #item | #(item) | (#(item)) | (#item)
pub fn fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>> {
    fn inner_fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>> {
        alt((
            simple_fragment_parser, // #item
            fragment_parser_with_optional_item, // #(item)
        ))(i)
    }
    alt((
        inner_fragment_parser, // #item | #(item)
        optional_matches_v(inner_fragment_parser) // (#(item)) | (#item)
    ))(i)
}

