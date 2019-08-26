use crate::new_parser::Token;
use crate::new_parser::{CaptureVariant, CaptureOrMatch};


#[derive(Debug, PartialEq)]
pub enum OptimizedToken {
    /// Extraneous section-related tokens can be condensed into a match.
    Match(String),
    Capture(CaptureVariant),
    Optional(Vec<OptimizedToken>)
}


impl From<CaptureOrMatch> for OptimizedToken {
    fn from(value: CaptureOrMatch) -> Self {
        match value {
            CaptureOrMatch::Match(m) => OptimizedToken::Match(m),
            CaptureOrMatch::Capture(v) => OptimizedToken::Capture(v)
        }
    }
}

impl OptimizedToken {
    /// Helper method to get concrete literals out of Match variants.
    pub fn lookup_next_concrete_sequence(&self) -> Result<&str, ()> {
        if let OptimizedToken::Match(sequence) = self {
            Ok(&sequence)
        } else {
            Err(())
        }
    }
}


/// Tokens that can be coalesced to a OptimizedToken::Match are converted to strings here.
fn token_to_string(token: &Token) -> &str {
    match token {
        Token::Separator => "/",
        Token::Match(literal) => &literal,
        Token::QueryBegin => "?",
        Token::QuerySeparator => "&",
        Token::FragmentBegin => "#",
        Token::Capture {..} | Token::QueryCapture {..} | Token::Optional(_)=> {
            unreachable!()
        }
    }
}


pub fn parse_str_and_optimize_tokens(i: &str) -> Result<Vec<OptimizedToken>, ()> {
    let (_, tokens) = crate::new_parser::parse(i).map_err(|_| ())?;
    Ok(optimize_tokens(tokens))
}

pub fn optimize_tokens(tokens: Vec<Token>) -> Vec<OptimizedToken> {
    // The list of optimized tokens.
    let mut optimized = vec![];
    // Stores consecutive Tokens that can be reduced down to a OptimizedToken::Match.
    let mut run = vec![];

    tokens.into_iter().for_each( |token| {
        match &token {
            Token::Separator | Token::Match(_) | Token::QueryBegin | Token::QuerySeparator | Token::FragmentBegin => {
                run.push(token)
            }
            Token::Optional(tokens) => {
                optimized.push(OptimizedToken::Optional(optimize_tokens(tokens.clone()))) // TODO, don't know if this is technically correct.
            },
            Token::Capture (_) | Token:: QueryCapture {..} => {
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(OptimizedToken::Match(s));
                    run.clear()
                }
                match token {
                    Token::Capture (variant) => {
                        optimized.push(OptimizedToken::Capture (variant))
                    },
                    Token::QueryCapture {ident, capture_or_match} => {
                        optimized.extend(vec![OptimizedToken::Match(format!("{}=", ident)), capture_or_match.into()])
                    }
                    _ => {
                        log::error!("crashing time");
                        unreachable!()
                    }
                };
            }
        }
    });
    // empty the "run"
    if !run.is_empty() {
        let s: String = run.iter().map(token_to_string).collect();
        optimized.push(OptimizedToken::Match(s));
    }
    optimized
}

