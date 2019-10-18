use crate::parser::core::{capture, match_exact};
use crate::parser::util::optional_matches;
use crate::parser::RouteParserToken;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::error::{context, VerboseError};
use nom::multi::many1;
use nom::sequence::pair;
use nom::IResult;

/// * /
/// * /item
/// * /item/item
/// * /item/item/item
/// * /item(/item)
/// * /item(/item)(/item)
/// * (/item)
/// * (/item)/item(/item)
pub fn path_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    fn inner_path_parser(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
        map(
            pair(separator_token, section_matchers),
            |(sep, sections)| {
                let mut x = Vec::with_capacity(sections.len() + 1);
                x.push(sep);
                x.extend(sections);
                x
            },
        )(i)
    }

    // /item/item/item
    let many_inner_paths = map(
        many1(inner_path_parser),
        |tokens: Vec<Vec<RouteParserToken>>| tokens.into_iter().flatten().collect::<Vec<_>>(),
    );

    // (/item)(/item)(/item)
    let many_optional_inner_paths = many1(optional_matches(inner_path_parser));

    let either_option_or_concrete = map(
        many1(alt((many_inner_paths, many_optional_inner_paths))),
        |x: Vec<Vec<RouteParserToken>>| x.into_iter().flatten().collect::<Vec<_>>(),
    );

    // accept any number of /thing or just '/
    context(
        "path parser",
        nom::combinator::verify(
            alt((
                map(
                    pair(either_option_or_concrete, opt(separator_token)),
                    |(mut paths, ending_separator)| {
                        paths.extend(ending_separator);
                        paths
                    },
                ),
                map(separator_token, |x| vec![x]),
            )),
            validate_path_parser_tokens,
        ),
    )(i)
}

/// The separator for paths sections.
fn separator_token(i: &str) -> IResult<&str, RouteParserToken, VerboseError<&str>> {
    context("/", map(char('/'), |_| RouteParserToken::Separator))(i)
}

pub fn section_matchers(i: &str) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
    let (i, token): (&str, RouteParserToken) =
        context("section matchers", alt((match_exact, capture)))(i)?;
    let tokens = vec![token];

    /// You can't have two matching sections in a row, because there is nothing to indicate when
    /// one ends and the other begins.
    /// This function collects possible section matchers and prevents them auto-glob matchers
    /// from residing next to each other.
    fn match_next_section_matchers(
        i: &str,
        mut tokens: Vec<RouteParserToken>,
    ) -> IResult<&str, Vec<RouteParserToken>, VerboseError<&str>> {
        let token = tokens.last().expect("Must be at least one token.");
        match token {
            RouteParserToken::Exact(_) => {
                let (i, t) = opt(capture)(i)?;
                if let Some(new_t) = t {
                    tokens.push(new_t);
                    match_next_section_matchers(i, tokens)
                } else {
                    Ok((i, tokens))
                }
            }
            RouteParserToken::Capture(_) => {
                let (i, t) = opt(match_exact)(i)?;
                if let Some(new_t) = t {
                    tokens.push(new_t);
                    match_next_section_matchers(i, tokens)
                } else {
                    Ok((i, tokens))
                }
            }
            _ => unreachable!(),
        }
    }

    match_next_section_matchers(i, tokens)
}

/// Makes sure that two capture groups can't be next to each other, even across optional boundaries.
///
/// The reason this is needed, is that by introducing optional sections,
/// the possibility exists that someone might create a `(/capture)capture` arrangement.
/// This shouldn't be allowed because due to the lack of an exact matcher between them,
/// the second capture will never get a chance to run.
fn validate_path_parser_tokens(tokens: &[RouteParserToken]) -> bool {
    let mut linearized_tokens = vec![];
    // Pull the tokens present in optional tokens into a single, non-nested collection of tokens.
    tokens.iter().for_each(|t| match t {
        RouteParserToken::Optional(inner) => linearized_tokens.extend(inner),
        token @ _ => linearized_tokens.push(token),
    });
    linearized_tokens.windows(2).all(|win| {
        match win {
            [RouteParserToken::Capture(_), RouteParserToken::Capture(_)] => false, // TODO because of this, would it be possible to relax the restriction around mandating path optionals start with '/'
            _ => true,
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Capture, CaptureVariant};
    use nom::combinator::all_consuming;
    use nom::error::ErrorKind;
    use nom::error::ErrorKind::Alt;
    use nom::error::ParseError;
    use nom::error::VerboseErrorKind::{Char, Context, Nom};
    use nom::Err;

    #[test]
    fn path_must_start_with_separator() {
        all_consuming(path_parser)("lorem").expect_err("Should reject at absence of /");
    }

    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_0() {
        let e = all_consuming(path_parser)("/path{}{match}").expect_err("Should not validate");
        assert_eq!(
            e,
            Err::Error(VerboseError::from_error_kind("{}{match}", ErrorKind::Eof))
        )
    }

    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_1() {
        let e =
            all_consuming(path_parser)("/path{match1}{match2}").expect_err("Should not validate");
        assert_eq!(
            e,
            Err::Error(VerboseError::from_error_kind("{match2}", ErrorKind::Eof))
        )
    }

    #[test]
    fn path_cant_contain_multiple_matches_in_a_row_2() {
        let e = all_consuming(path_parser)("/path{}{}").expect_err("Should not validate");
        assert_eq!(
            e,
            Err::Error(VerboseError::from_error_kind("{}{}", ErrorKind::Eof))
        )
    }

    #[test]
    fn section_matchers_falis_to_match() {
        let e = section_matchers("{aoeu").expect_err("Should not complete");

        let error = VerboseError {
            errors: vec![
                ("", Char('}')),
                ("{aoeu", Context("capture")),
                ("{aoeu", Nom(Alt)),
                ("{aoeu", Context("section matchers")),
            ],
        };
        assert_eq!(e, Err::Error(error));
    }

    #[test]
    fn cant_have_double_slash() {
        all_consuming(path_parser)("//)").expect_err("Should not validate");
    }

    #[test]
    fn option_section() {
        path_parser("/lorem(/ipsum)").expect("Should validate");
    }

    #[test]
    fn option_section_with_trailing_sep() {
        path_parser("/lorem(/ipsum)/").expect("Should validate");
    }

    #[test]
    fn many_option_section() {
        let (_, tokens) = path_parser("/first[/second][/third]").expect("Should validate");
        let expected = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("first".to_string()),
            RouteParserToken::Optional(vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("second".to_string()),
            ]),
            RouteParserToken::Optional(vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("third".to_string()),
            ]),
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn option_section_between_exacts() {
        let (_, tokens) = path_parser("/first[/second]/third").expect("Should validate");
        let expected = vec![
            RouteParserToken::Separator,
            RouteParserToken::Exact("first".to_string()),
            RouteParserToken::Optional(vec![
                RouteParserToken::Separator,
                RouteParserToken::Exact("second".to_string()),
            ]),
            RouteParserToken::Separator,
            RouteParserToken::Exact("third".to_string()),
        ];
        assert_eq!(tokens, expected);
    }



    #[test]
    fn option_section_between_captures_fails() {
        all_consuming(path_parser)("/first[/{}]{}").expect_err("Should not validate");
    }

    #[test]
    fn option_section_can_start_matcher_string() {
        path_parser("[/lorem]").expect("Should validate");
    }
}
