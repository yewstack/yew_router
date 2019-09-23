use nom::bytes::complete::{tag, tag_no_case};
use nom::combinator::{cond, map};
use nom::sequence::pair;
use nom::IResult;

/// Allows a configurable tag that can optionally be case insensitive.
pub fn tag_possibly_case_sensitive<'a, 'b: 'a>(
    text: &'b str,
    is_sensitive: bool,
) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    map(
        pair(
            cond(is_sensitive, tag(text)),
            cond(!is_sensitive, tag_no_case(text)),
        ),
        |(x, y): (Option<&str>, Option<&str>)| x.xor(y).unwrap(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn case_sensitive() {
        let parser = tag_possibly_case_sensitive("hello", true);
        parser("hello").expect("Should match");
        parser("HeLLo").expect_err("Should not match");
    }

    #[test]
    fn case_insensitive() {
        let parser = tag_possibly_case_sensitive("hello", false);
        parser("hello").expect("Should match");
        parser("HeLLo").expect("Should match");
    }

}
