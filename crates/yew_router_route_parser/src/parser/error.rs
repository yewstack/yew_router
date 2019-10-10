//! Error handling.
use crate::parser::core::valid_exact_match_characters;
use crate::parser::util::skip_until;
use core::fmt::Write;
use nom::branch::alt;
use nom::character::complete::char;
use nom::error::VerboseError;
use nom::multi::many0_count;
use nom::sequence::terminated;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};
use ExpectedConstruct as Ec;

const DOUBLE_SLASHES_NOT_ALLOWED: &str = "Double slashes ('//') are not allowed.";
const EMPTY_MATCH_NOT_ALLOWED: &str =
    "Empty match strings are not allowed. You are allowed to match anything by specifying '{}'.";
const CAPTURE_BLOCK_LONG: &str = "A capture block can be made up of: '{}', '{<ident>}', '{*}', '{*:<ident>}', '{<number>}', or '{<number>:<ident>}'. The indicated character does not fit into one of these patterns.";
const CAPTURE_BLOCK_SHORT: &str = "A capture block can be made up of: '{}', '{<ident>}', '{*}', '{*:<ident>}', '{<number>}', or '{<number>:<ident>}'.";
const SECONDARY_QUERIES_USE_AND: &str =
    "Secondary queries should be started with '&' instead of '?'.";
const UNCLOSED_OPTIONAL: &str = "There are more open parenthesis than close parenthesis. There must be the same number of open parenthesis as close parenthesis.";
const TOO_MANY_OPTIONAL_CLOSES: &str = "There are more close parenthesis than open parenthesis. There must be the same number of open parenthesis as close parenthesis.";
const DANGLING_QUERY: &str = "Dangling query. Queries exist in the form '<?|&><exact>=<query|exact>'. The '=<query|exact>' section was left off here.";
const UNHANDLED_ERROR: &str = "Unhandled error.";

/// A struct to hold information for printing a useful error message to a user for their parser.
#[derive(Default, PartialEq)]
pub struct YewRouterParseError<'a> {
    input: &'a str,
    offset: usize,
    expected: Vec<ExpectedConstruct>,
    reason: String,
}

#[derive(Debug, PartialEq)]
pub enum ExpectedConstruct {
    Slash,
    Question,
    And,
    Equals,
    Hash,
    OpenBrace,
    CloseBrace,
    OpenParen,
    #[allow(unused)]
    CloseParen,
    StarColonIdent,
    Star,
    NumberColonIdent,
    Number,
    ValidIdent,
    ExactText,
}

impl Display for ExpectedConstruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Ec::Slash => f.write_str("'/'"),
            ExpectedConstruct::Question => f.write_str("'?'"),
            ExpectedConstruct::And => f.write_str("'&'"),
            ExpectedConstruct::Equals => f.write_str("'='"),
            ExpectedConstruct::Hash => f.write_str("'#'"),
            ExpectedConstruct::OpenBrace => f.write_str("'{'"),
            ExpectedConstruct::CloseBrace => f.write_str("'}'"),
            ExpectedConstruct::OpenParen => f.write_str("'('"),
            ExpectedConstruct::CloseParen => f.write_str("')'"),
            ExpectedConstruct::ValidIdent => f.write_str("<Identifier>"),
            ExpectedConstruct::ExactText => f.write_str("<Exact Text>"),
            ExpectedConstruct::StarColonIdent => f.write_str("*:<Ident>"),
            ExpectedConstruct::Star => f.write_str("*"),
            ExpectedConstruct::NumberColonIdent => f.write_str("<number>:<Ident>"),
            ExpectedConstruct::Number => f.write_str("<number>"),
        }
    }
}

/// Simple offset calculator to determine where to place the carrot for indicating an error.
fn offset(input: &str, substring: &str) -> usize {
    input.len() - substring.len()
}

impl<'a> YewRouterParseError<'a> {
    /// From Nom's verbose error type.
    pub fn from_verbose_error(input: &'a str, err: VerboseError<&'a str>) -> Self {
        let (substring, _kind) = err.errors.first().unwrap();
        let mut offset: usize = offset(input, substring);

        let (expected, reason): (Vec<Ec>, String) = if double_slash(input, substring, offset) {
            (
                vec![
                    Ec::ExactText,
                    Ec::OpenBrace,
                    Ec::OpenParen,
                    Ec::Question,
                    Ec::Hash,
                ],
                DOUBLE_SLASHES_NOT_ALLOWED.to_string(),
            )
        } else if bad_capture(substring) {
            let new_offset = find_bad_capture_character_offset(offset, substring);
            // If the new offset doesn't point to the first char inside the capture block,
            // it should be assumed that they should just close the capture.
            // TODO its possible to get higher fidelity of what characters are acceptable here, but for now this is fine.
            // TODO doesn't handle double ** or ::
            if new_offset > offset + 1 {
                offset = new_offset;
                (vec![Ec::CloseBrace], CAPTURE_BLOCK_LONG.to_string())
            } else {
                offset = new_offset;
                (
                    vec![
                        Ec::ValidIdent,
                        Ec::CloseBrace,
                        Ec::Number,
                        Ec::NumberColonIdent,
                        Ec::Star,
                        Ec::StarColonIdent,
                    ],
                    CAPTURE_BLOCK_SHORT.to_string(),
                )
            }
        } else if multiple_query_beginnings(input, substring) {
            (vec![Ec::And], SECONDARY_QUERIES_USE_AND.to_string())
        } else if unclosed_optional(input) {
            (vec![], UNCLOSED_OPTIONAL.to_string())
        } else if too_many_closed_optional(input) {
            (vec![], TOO_MANY_OPTIONAL_CLOSES.to_string())
        } else if dangling_query(substring) {
            offset = dangling_query_offset(input);
            (vec![Ec::Equals], DANGLING_QUERY.to_string())
        } else if offset == 0 {
            (
                vec![Ec::Slash, Ec::Question, Ec::Hash, Ec::OpenBrace],
                EMPTY_MATCH_NOT_ALLOWED.to_string(),
            )
        } else {
            (vec![], UNHANDLED_ERROR.to_string())
        };

        YewRouterParseError {
            input,
            offset,
            expected,
            reason,
        }
    }

    /// From Nom's `Err` type.
    ///
    /// Makes the assumption that an Error variant is always returned.
    pub fn from_err(input: &'a str, err: nom::Err<VerboseError<&'a str>>) -> Option<Self> {
        match err {
            nom::Err::Error(err) => Some(YewRouterParseError::from_verbose_error(input, err)),
            _ => None,
        }
    }
}

impl<'a> Display for YewRouterParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_char('\n')?;
        f.write_str(self.input)?;
        f.write_char('\n')?;
        let pad = (0..self.offset).map(|_| '-').collect::<String>();
        f.write_str(&format!("{}^", pad))?;
        if !self.expected.is_empty() {
            f.write_char('\n')?;
            let expected: String = self
                .expected
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<String>>()
                .join(", ");
            f.write_str(&format!("Expected one of: {}.", expected))?;
        }
        f.write_char('\n')?;
        f.write_str(&format!("Message:         '{}'", self.reason))
    }
}

// TODO eventually switch back to a normal debug, then make sure that the panic produced by the macro uses the display.
impl<'a> Debug for YewRouterParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        // Delegate the debug impl to display.
        Display::fmt(self, f)
    }
}

/// Finds an instance of a '//'.
fn double_slash(input: &str, substring: &str, offset: usize) -> bool {
    input != substring
        && offset > 0
        && input.chars().nth(offset - 1) == Some('/') // Check the character in the input prior to the beginning of the substring
        && substring.starts_with('/')
}

/// Finds a capture that contains an invalid character.
fn bad_capture(substring: &str) -> bool {
    substring.starts_with('{')
        && substring
            .chars()
            .skip(1) // Skip the first, because that should be the opening '{'.
            .any(contains_forbidden_capture_character)
}

fn contains_forbidden_capture_character(c: char) -> bool {
    const INVALID_CHARACTERS: &str = "`~\"',.;!@#$%^&*()[]<>{/\\\n\t ";
    INVALID_CHARACTERS.contains(c)
}

/// Finds the offset of the bad character in the input, using the substring, bad character list, and existing offset.
fn find_bad_capture_character_offset(offset: usize, substring: &str) -> usize {
    let substr_index_of_invalid_char = substring
        .char_indices()
        .skip(1) // Skip the first, because that should be the opening '{'
        .filter_map(|(index, c)| {
            if contains_forbidden_capture_character(c) {
                Some(index)
            } else {
                None
            }
        })
        .next()
        .unwrap(); // One of these must contain the invalid char,
                   // given that this is called after bad_capture returns true
    offset + substr_index_of_invalid_char
}

/// Returns true if the query starts twice (denoted by a ?)
fn multiple_query_beginnings(input: &str, substring: &str) -> bool {
    use crate::parser::query::begin_query_parser;

    // Count the number of occurrences of the begin_query_parser appear.
    // If it is greater than 1, then the matcher string is misconstrued.
    let multiple_begin_queries =
        if let Ok((_, count)) = many0_count(skip_until(begin_query_parser))(input) {
            count > 1
        } else {
            false
        };

    substring.starts_with('?') // This check should help to avoid colliding with '(?...)' sections
        && multiple_begin_queries
}

/// Detects if there are more open than closed parenthesis.
fn unclosed_optional(input: &str) -> bool {
    if let Ok((_, open_count)) = many0_count(skip_until::<_, _, (), _>(char('[')))(input) {
        if let Ok((_, close_count)) = many0_count(skip_until::<_, _, (), _>(char(']')))(input) {
            open_count > close_count
        } else {
            false
        }
    } else {
        false
    }
}

/// Detects if there are more closed than open parenthesis.
fn too_many_closed_optional(input: &str) -> bool {
    if let Ok((_, open_count)) = many0_count(skip_until::<_, _, (), _>(char('[')))(input) {
        if let Ok((_, close_count)) = many0_count(skip_until::<_, _, (), _>(char(']')))(input) {
            open_count < close_count
        } else {
            false
        }
    } else {
        false
    }
}

// Detects if there was an instance of a query that didn't have an =<capture|match> present
fn dangling_query(substring: &str) -> bool {
    match alt((
        terminated(skip_until(char('?')), valid_exact_match_characters),
        terminated(skip_until(char('&')), valid_exact_match_characters),
    ))(substring)
    {
        Ok((rest, _)) => rest.is_empty() || rest.starts_with('&') || rest.starts_with('#'),
        Err(x) => {
            dbg!(x);
            false
        }
    }
}

fn dangling_query_offset(input: &str) -> usize {
    let (rest, _) = alt((
        terminated(skip_until(char('?')), valid_exact_match_characters),
        terminated(skip_until(char('&')), valid_exact_match_characters),
    ))(input)
    .unwrap();

    if rest.is_empty() {
        input.len()
    } else {
        input.len() - rest.len()
    }
}

#[cfg(test)]
mod test_conditions {
    use super::*;
    #[test]
    fn double_slash_true() {
        assert!(double_slash("//lorem", "/lorem", 1))
    }

    #[test]
    fn double_slash_true_in_later_substring() {
        assert!(double_slash("/lorem//ipsum", "/ipsum", 7))
    }

    #[test]
    fn double_slash_reject_same() {
        assert!(!double_slash("/lorem", "/lorem", 0))
    }

    #[test]
    fn double_slash_reject_displaced_substring() {
        assert!(!double_slash("/lorem/ipsum", "/ipsum", 6))
    }

    // ----------------

    #[test]
    fn bad_capture_first_character_reject() {
        let substring = "{}";
        assert!(!bad_capture(substring))
    }

    #[test]
    fn bad_capture_first_character_true() {
        assert!(bad_capture("{{}}"))
    }

    #[test]
    fn bad_capture_bad_char_after_ident_true() {
        assert!(bad_capture("{identifier(}"))
    }

    // -------------------

    #[test]
    fn multiple_query_beginnings_test() {
        assert!(multiple_query_beginnings(
            "?lorem=ipsum&dolor=sit?amet=consectetur",
            "?amet=consectetur"
        ));
    }

    #[test]
    fn multiple_query_beginnings_avoids_false_positive() {
        assert!(!multiple_query_beginnings(
            "(?lorem=ipsum)(?dolor=sit)",
            "(?dolor=sit)"
        ));
    }

    // ------
    #[test]
    fn dangling_query_detected() {
        assert!(dangling_query("/yeet/?thing"))
    }

    #[test]
    fn dangling_query_detected_with_valid_query_after() {
        assert!(dangling_query("/yeet/?thing&query=something"))
    }

    #[test]
    fn dangling_query_detected_with_fragment_after() {
        assert!(dangling_query("/yeet/?thing#fragment"))
    }

    #[test]
    fn dangling_query_avoids_false_positive_literal() {
        assert!(!dangling_query("?thing=yes"))
    }

    #[test]
    fn dangling_query_avoids_false_positive_capture() {
        assert!(!dangling_query("?thing={}"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn double_slash_error() {
        let input = "/lorem//ipsum";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 7,
            expected: vec![
                Ec::ExactText,
                Ec::OpenBrace,
                Ec::OpenParen,
                Ec::Question,
                Ec::Hash,
            ],
            reason: DOUBLE_SLASHES_NOT_ALLOWED.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn double_slash_error_displays_correctly() {
        let input = "/lorem//ipsum";
        let error = parse(input).expect_err("should fail");
        let printed_error = format!("{}", error);
        let expected = r##"
/lorem//ipsum
-------^
Expected one of: <Exact Text>, '{', '(', '?', '#'.
Message:         'Double slashes ('//') are not allowed.'"##;
        assert_eq!(printed_error, expected);
    }

    #[test]
    fn double_slash_simple_error() {
        let input = "//";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 1,
            expected: vec![
                Ec::ExactText,
                Ec::OpenBrace,
                Ec::OpenParen,
                Ec::Question,
                Ec::Hash,
            ],
            reason: DOUBLE_SLASHES_NOT_ALLOWED.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn empty_error() {
        let input = "";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 0,
            expected: vec![Ec::Slash, Ec::Question, Ec::Hash, Ec::OpenBrace],
            reason: EMPTY_MATCH_NOT_ALLOWED.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn nested_capture_error() {
        let input = "/lorem/{{}}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 8,
            expected: vec![
                Ec::ValidIdent,
                Ec::CloseBrace,
                Ec::Number,
                Ec::NumberColonIdent,
                Ec::Star,
                Ec::StarColonIdent,
            ],
            reason: CAPTURE_BLOCK_SHORT.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn malformed_capture_error() {
        let input = "/lorem/{/}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 8,
            expected: vec![
                Ec::ValidIdent,
                Ec::CloseBrace,
                Ec::Number,
                Ec::NumberColonIdent,
                Ec::Star,
                Ec::StarColonIdent,
            ],
            reason: CAPTURE_BLOCK_SHORT.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn capture_malformed_after_ident_error() {
        let input = "/lorem/{ident/}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 13,
            expected: vec![Ec::CloseBrace],
            reason: CAPTURE_BLOCK_LONG.to_string(),
        };
        assert_eq!(error, expected)
    }

    // ---------------

    #[test]
    fn multiple_queries() {
        let input = "?this=that?what=whom";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 10,
            expected: vec![Ec::And],
            reason: SECONDARY_QUERIES_USE_AND.to_string(),
        };
        assert_eq!(error, expected)
    }

    // --------------
    #[test]
    fn too_many_open_parens() {
        let input = "[/thing][/other][[/thing]";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 16,
            expected: vec![],
            reason: UNCLOSED_OPTIONAL.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn too_many_close_parens() {
        let input = "[/thing][/other][/thing]]";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 24,
            expected: vec![],
            reason: TOO_MANY_OPTIONAL_CLOSES.to_string(),
        };
        assert_eq!(error, expected)
    }

    // --------------

    #[test]
    fn dangling_query_terminating() {
        let input = "?bad_query";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 10,
            expected: vec![Ec::Equals],
            reason: DANGLING_QUERY.to_string(),
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn dangling_query_before_valid_query() {
        let input = "?bad_query&query=thing";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 10,
            expected: vec![Ec::Equals],
            reason: DANGLING_QUERY.to_string(),
        };
        assert_eq!(error, expected)
    }
}
