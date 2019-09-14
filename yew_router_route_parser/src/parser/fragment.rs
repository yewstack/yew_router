use crate::parser::path::section_matchers; // TODO possibly duplicate this function (loosen its requirements for this module eg. allow '/' characters.)
use crate::parser::util::vectorize;
use crate::parser::util::{optional_matches, optional_matches_v};
use crate::parser::RouteParserToken;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::error::{context, VerboseError};
use nom::IResult;

fn begin_fragment_token(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    map(char('#'), |_| RouteParserToken::FragmentBegin)(i)
}

/// #item
fn simple_fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, mut section) = section_matchers(i)?;
    let mut v = vec![begin];
    v.append(&mut section);
    Ok((i, v))
}

/// #(item)
fn fragment_parser_with_optional_item(
    i: &str,
) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, optional) = optional_matches(section_matchers)(i)?;
    let v = vec![begin, optional];
    Ok((i, v))
}

/// #item | #(item) | (#(item)) | (#item)
pub fn fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    fn inner_fragment_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
        alt((
            simple_fragment_parser,             // #item
            fragment_parser_with_optional_item, // #(item)
            vectorize(begin_fragment_token),    // #
        ))(i)
    }
    context(
        "fragment",
        alt((
            inner_fragment_parser,                     // #item | #(item)
            optional_matches_v(inner_fragment_parser), // (#(item)) | (#item)
        )),
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_fragment() {
        fragment_parser("#").expect("should parse");
    }

    #[test]
    fn simple_fragment() {
        fragment_parser("#hello").expect("should parse");
    }

    #[test]
    fn optional_fragment() {
        fragment_parser("#(hello)").expect("should parse");
    }

    #[test]
    fn entirely_optional_simple_fragment() {
        fragment_parser("(#)").expect("should parse");
    }

    #[test]
    fn entirely_optional_fragment() {
        fragment_parser("(#hello)").expect("should parse");
    }
}
