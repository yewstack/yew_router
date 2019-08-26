use nom::IResult;
use nom::combinator::peek;
use nom::character::is_digit;
use nom::error::ErrorKind;
use nom::bytes::complete::{is_not, take};

/// Captures a string up to the point where a character not possible to be present in Rust's identifier is encountered.
/// It prevents the first character from being a digit.
pub fn valid_ident_characters(i: &str) -> IResult<&str, &str> {
    const INVALID_CHARACTERS: &str = " -*/+#?&^@~`;,.|\\{}[]()=\t\n";
    let (i, next) = peek(take(1usize))(i)?; // Look at the first character
    if is_digit(next.bytes().next().unwrap()) {
        return Err(nom::Err::Error((i, ErrorKind::Digit))) // Digits not allowed
    } else {
        is_not(INVALID_CHARACTERS)(i)
    }
}
