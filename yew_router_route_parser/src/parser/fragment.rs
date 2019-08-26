use nom::IResult;
use crate::parser::Token;
use crate::parser::optional_section;
use crate::parser::ret_vec;
use crate::parser::begin_fragment_token;
use crate::parser::section_matchers;
use nom::branch::alt;

/// #item
fn simple_fragment_parser(i: &str) -> IResult<&str, Vec<Token>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, mut section) = section_matchers(i)?;
    let mut v = vec![begin];
    v.append(&mut section);
    Ok((i, v))
}

/// #(item)
fn fragment_parser_with_optional_item(i: &str) -> IResult<&str, Vec<Token>> {
    let (i, begin) = begin_fragment_token(i)?;
    let (i, optional) = optional_section(section_matchers)(i)?;
    let v = vec![begin, optional];
    Ok((i, v))
}

pub(super) fn fragment_parser(i: &str) -> IResult<&str, Vec<Token>> {
    fn inner_fragment_parser(i: &str) -> IResult<&str, Vec<Token>> {
        alt((
            simple_fragment_parser, // #item
            fragment_parser_with_optional_item, // #(item)
        ))(i)
    }
    alt((
        inner_fragment_parser, // #item | #(item)
        ret_vec(optional_section(inner_fragment_parser)) // (#(item)) | (#item)
    ))(i)
}

