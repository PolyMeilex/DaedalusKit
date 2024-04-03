#![allow(clippy::single_match)]

pub use daedalus_lexer::DaedalusLexer;
pub mod fmt;
mod parse;

pub use parse::*;

pub type ParseError = daedalus_lexer::TokenError;
pub type ParseErrorKind = daedalus_lexer::TokenErrorKind;
