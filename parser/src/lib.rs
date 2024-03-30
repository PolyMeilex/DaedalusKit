#![allow(clippy::single_match)]

pub use lexer::DaedalusLexer;
pub mod fmt;
pub mod parse;

pub type ParseError = lexer::TokenError;
pub type ParseErrorKind = lexer::TokenErrorKind;
