//! Parser for a "matcher string". The tokens produced by this parser are used to construct a
//! matcher.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_qualifications
)]

// pub mod parser_old;
pub mod parser;

// pub use parser_old::{Capture, CaptureVariant};
pub use parser::{parse_str_and_optimize_tokens, ParserError};
use std::collections::HashMap;
// pub use token_optimizer::{parse_str_and_optimize_tokens, CaptureVariant, MatcherToken};

/// Captures contain keys corresponding to named match sections,
/// and values containing the content captured by those sections.
pub type Captures<'a> = HashMap<&'a str, String>;


/// Tokens used to determine how to match and capture sections from a URL.
#[derive(Debug, PartialEq, Clone)]
pub enum MatcherToken {
    /// Section-related tokens can be condensed into a match.
    Exact(String),
    /// Capture section.
    Capture(CaptureVariant),
}

/// Variants that indicate how part of a string should be captured.
#[derive(Debug, PartialEq, Clone)]
pub enum CaptureVariant {
    /// {name} - captures a section and adds it to the map with a given name.
    Named(String),
    /// {*:name} - captures over many sections and adds it to the map with a given name.
    ManyNamed(String),
    /// {2:name} - captures a fixed number of sections with a given name.
    NumberedNamed {
        /// Number of sections to match.
        sections: usize,
        /// The key to be entered in the `Matches` map.
        name: String,
    },
}
