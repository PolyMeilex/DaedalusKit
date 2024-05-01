#![allow(clippy::single_match)]

use std::num::{ParseFloatError, ParseIntError};

pub use daedalus_lexer as lexer;
pub use daedalus_lexer::DaedalusLexer;
mod parse;

use logos::Span;
pub use parse::*;

pub type Result<T> = std::result::Result<T, ParseError>;

type ParseBacktrace = std::backtrace::Backtrace;

pub struct DaedalusParser<'a> {
    pub lexer: &'a mut DaedalusLexer<'a>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    TokenError(#[from] daedalus_lexer::TokenError),
    #[error("{err}")]
    IntLitError {
        err: ParseIntError,
        span: Span,
        backtrace: ParseBacktrace,
    },
    #[error("{err}")]
    FloatLitError {
        err: ParseFloatError,
        span: Span,
        backtrace: ParseBacktrace,
    },
}

impl ParseError {
    pub fn span(&self) -> &Span {
        match self {
            ParseError::TokenError(err) => err.span(),
            ParseError::FloatLitError { span, .. } => span,
            ParseError::IntLitError { span, .. } => span,
        }
    }

    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        match self {
            ParseError::TokenError(err) => err.backtrace(),
            ParseError::FloatLitError { backtrace, .. } => backtrace,
            ParseError::IntLitError { backtrace, .. } => backtrace,
        }
    }
}

pub type ParseErrorKind = daedalus_lexer::TokenErrorKind;
