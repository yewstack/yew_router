//! Error handling.
use nom::error::VerboseError;
use nom::Offset;
use ExpectedConstruct as Ec;
use std::fmt::{Display, Formatter, Error as FmtError, Debug};
use core::fmt::Write;

const DOUBLE_SLASHES_NOT_ALLOWED: &str = "Double slashes ('//') are not allowed.";
const EMPTY_MATCH_NOT_ALLOWED: &str = "Empty match strings are not allowed. You are allowed to match anything by specifying '{}'.";
const CAPTURE_BLOCK_LONG: &str = "A capture block can be made up of: '{}', '{<ident>}', '{*}', '{*:<ident>}', '{<number>}', or '{<number>:<ident>}'. The indicated character does not fit into one of these patterns.";
const CAPTURE_BLOCK_SHORT: &str = "A capture block can be made up of: '{}', '{<ident>}', '{*}', '{*:<ident>}', '{<number>}', or '{<number>:<ident>}'.";

/// A struct to hold information for printing a useful error message to a user for their parser.
#[derive(Default, PartialEq)]
pub struct YewRouterParseError<'a> {
    input: &'a str,
    offset: usize,
    expected: Vec<ExpectedConstruct>,
    reason: Option<String>
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
    CloseParen,
    StarColonIdent,
    Star,
    NumberColonIdent,
    Number,
    ValidIdent,
    ExactText,
    UnhandledError
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
            ExpectedConstruct::UnhandledError => f.write_str("Unhandled Error"),
            ExpectedConstruct::StarColonIdent=> f.write_str("*:<Identifier>"),
            ExpectedConstruct::Star => f.write_str("*"),
            ExpectedConstruct::NumberColonIdent => f.write_str("<number>:<Identifier>"),
            ExpectedConstruct::Number => f.write_str("<number>")
        }
    }
}



fn offset(input: &str, substring:&str) -> usize {
    input.len() - substring.len()
}

impl <'a> YewRouterParseError<'a> {
    /// From Nom's verbose error type.
    pub fn from_verbose_error(input: &'a str, err: VerboseError<&'a str>) -> Self {
        let (substring, _kind) = err.errors.first().unwrap();
        let mut offset: usize = offset(input, substring);

        let (expected, reason): (Vec<Ec>, Option<String>) = if offset == 0 {
            (
                vec![Ec::Slash, Ec::Question, Ec::Hash, Ec::OpenBrace],
                Some(EMPTY_MATCH_NOT_ALLOWED.to_string())
            )
        } else if double_slash(input, substring, offset)  {
            (
                vec![Ec::ExactText, Ec::OpenBrace, Ec::OpenParen, Ec::Question, Ec::Hash],
                Some(DOUBLE_SLASHES_NOT_ALLOWED.to_string())
            )
        } else if bad_capture(input, substring, offset){
            let new_offset = find_bad_capture_character_offset(offset, substring);
            // If the new offset doesn't point to the first char inside the capture block,
            // it should be assumed that they should just close the capture.
            // TODO its possible to get higher fidelity of what characters are acceptable here, but for now this is fine.
            // TODO doesn't handle double ** or ::
            if new_offset > offset + 1 {
                offset = new_offset;
                (
                    vec![Ec::CloseBrace],
                    Some(CAPTURE_BLOCK_LONG.to_string())
                )
            } else {
                offset = new_offset;
                (
                    vec![Ec::ValidIdent, Ec::CloseBrace, Ec::Number, Ec::NumberColonIdent, Ec::Star, Ec::StarColonIdent],
                    Some(CAPTURE_BLOCK_SHORT.to_string())
                )
            }
        } else {
            (vec![Ec::UnhandledError], None)
        };

        YewRouterParseError {
            input,
            offset,
            expected,
            reason
        }
    }

    /// From Nom's `Err` type.
    ///
    /// Makes the assumption that an Error variant is always returned.
    pub fn from_err(input: &'a str, err: nom::Err<VerboseError<&'a str>>) -> Option<Self> {
        match err {
            nom::Err::Error(err) => {
                 Some(YewRouterParseError::from_verbose_error(input, err))
            }
            _ => None
        }
    }
}

impl <'a> Display for YewRouterParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_char('\n')?;
        f.write_str(self.input)?;
        f.write_char('\n')?;
        let pad = (0..self.offset).map(|_| '-').collect::<String>();
        f.write_str(&format!("{}^", pad))?;
        f.write_char('\n')?;
        let expected: String = self.expected.iter().map(|e| format!("{}", e)).collect::<Vec<String>>().join(", ");
        f.write_str(&format!("Expected one of: {}.", expected))?;
        if let Some(reason) = &self.reason {
            f.write_char('\n');
            f.write_str(&format!("Message:         '{}'", reason))
        } else {
            Ok(())
        }
    }
}

impl <'a> Debug for YewRouterParseError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        // Delegate the debug impl to display.
        Display::fmt(self, f)
    }
}


fn double_slash(input: &str, substring: &str, offset: usize) -> bool {
    input != substring
        && offset > 0
        && input.chars().nth(offset - 1) == Some('/') // Check the character in the input prior to the beginning of the substring
        && substring.chars().next() == Some('/')
}

fn bad_capture(input: &str, substring: &str, offset: usize) -> bool {
    input != substring
        && input.chars().nth(offset ) == Some('{')
        && substring.chars().skip(1).any(contains_forbidden_capture_character)
}


fn contains_forbidden_capture_character(c: char) -> bool {
    println!("contains forbidden char: {}", c);
    let invalid_characters = "!@#$%^&*()[]<>{/\\\n\t "; // TODO this should be made a constant somewhere
    invalid_characters.contains(c)
}

/// Finds the offset of the bad character in the input, using the substring, bad character list, and existing offset.
fn find_bad_capture_character_offset(offset: usize, substring: &str) -> usize {
    let substr_index_of_invalid_char = substring.char_indices()
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

    println!("{}", substring);
    println!("{}", substr_index_of_invalid_char);

    offset + substr_index_of_invalid_char
}



#[cfg(test)]
mod test_conditions {
    use super::*;
    #[test]
    fn double_slash_true() {
        assert!(double_slash("//hello", "/hello", 1))
    }

    #[test]
    fn double_slash_true_in_later_substring() {
        assert!(double_slash("/hello//there", "/there", 7))
    }

    #[test]
    fn double_slash_reject_same() {
        assert!(!double_slash("/hello", "/hello", 0))
    }

    #[test]
    fn double_slash_reject_displaced_substring() {
        assert!(!double_slash("/hello/there", "/there", 6))
    }

    // ----------------


    #[test]
    fn bad_capture_first_character_reject() {
        let input = "/hello/{}";
        let substring = "{}";
        let offset = offset(input, substring);
        assert!(!bad_capture(input, substring, offset))
    }

    #[test]
    fn bad_capture_first_character_true() {
        assert!(bad_capture("/hello/{{}}", "{{}}", 7))
    }

    #[test]
    fn bad_capture_bad_char_after_ident_true() {
        assert!(bad_capture("/hello/{identifier(}", "{identifier(}", 7))
    }


}


#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parse;
    use nom::Err as NomErr;



    #[test]
    fn double_slash_error() {
        let input = "/hello//there";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 7,
            expected: vec![Ec::ExactText, Ec::OpenBrace, Ec::OpenParen, Ec::Question, Ec::Hash],
            reason: Some(DOUBLE_SLASHES_NOT_ALLOWED.to_string())
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn double_slash_error_displays_correctly() {
        let input = "/hello//there";
        let error = parse(input).expect_err("should fail");
        let printed_error = format!("{}", error);
        let expected =
            r##"
/hello//there
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
            expected: vec![Ec::ExactText, Ec::OpenBrace, Ec::OpenParen, Ec::Question, Ec::Hash],
            reason: Some(DOUBLE_SLASHES_NOT_ALLOWED.to_string())
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
            reason: Some(EMPTY_MATCH_NOT_ALLOWED.to_string())
        };
        assert_eq!(error, expected)
    }


    #[test]
    fn nested_capture_error() {
        let input = "/hello/{{}}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 8,
            expected: vec![Ec::ValidIdent, Ec::CloseBrace, Ec::Number, Ec::NumberColonIdent, Ec::Star, Ec::StarColonIdent],
            reason: Some(CAPTURE_BLOCK_SHORT.to_string())
        };
        assert_eq!(error, expected)
    }

    #[test]
    fn malformed_capture_error() {
        let input = "/hello/{/}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 8,
            expected: vec![Ec::ValidIdent, Ec::CloseBrace, Ec::Number, Ec::NumberColonIdent, Ec::Star, Ec::StarColonIdent],
            reason: Some(CAPTURE_BLOCK_SHORT.to_string())
        };
        assert_eq!(error, expected)
    }


    #[test]
    fn capture_malformed_after_ident_error() {
        let input = "/hello/{ident/}";
        let error = parse(input).expect_err("should fail");

        let expected = YewRouterParseError {
            input,
            offset: 13,
            expected: vec![Ec::CloseBrace],
            reason: Some(CAPTURE_BLOCK_LONG.to_string())
        };
        assert_eq!(error, expected)
    }

}
