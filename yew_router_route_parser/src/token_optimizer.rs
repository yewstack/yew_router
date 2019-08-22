use crate::new_parser::Token;
use crate::new_parser::CaptureVariants;


#[derive(Debug, PartialEq)]
pub enum OptimizedToken {
    /// Extraneous section-related tokens can be condensed into a match.
    Match(String),
    Capture(CaptureVariants),
    QueryCapture {
        ident: String,
        value: String
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


fn token_to_string(token: &Token) -> &str {
    match token {
        Token::Separator => "/",
        Token::Match(literal) => &literal,
        Token::QueryBegin => "?",
        Token::QuerySeparator => "&",
        Token::FragmentBegin => "#",
        Token::Capture {..} | Token::QueryCapture {..} => {
//            log::error!("Bout to crash!");
            unreachable!()
        }
    }
}


pub fn parse_str_and_optimize_tokens(i: &str) -> Result<Vec<OptimizedToken>, ()> {
    let (_, tokens) = crate::new_parser::parse(i).map_err(|_| ())?;
    Ok(optimize_tokens(tokens))
}

pub fn optimize_tokens(tokens: Vec<Token>) -> Vec<OptimizedToken> {
    let mut optimized = vec![];
    let mut run = vec![];

    tokens.into_iter().for_each( |token| {
        match &token {
            Token::Separator | Token::Match(_) | Token::QueryBegin | Token::QuerySeparator | Token::FragmentBegin => {
                run.push(token)
            }
            Token::Capture (_) | Token:: QueryCapture {..} => {
                if !run.is_empty() {
                    let s: String = run.iter().map(token_to_string).collect();
                    optimized.push(OptimizedToken::Match(s));
                    run.clear()
                }
                let token = match token {
                    Token::Capture (variant) => OptimizedToken::Capture (variant),
                    Token::QueryCapture {ident, value} => OptimizedToken::QueryCapture {ident, value},
                    _ => {
                        log::error!("crashing time");
                        unreachable!()
                    }
                };
                optimized.push(token);
            }
        }
    });
    if !run.is_empty() {
        let s: String = run.iter().map(token_to_string).collect();
        optimized.push(OptimizedToken::Match(s));
    }
    optimized
}

