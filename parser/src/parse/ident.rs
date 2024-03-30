use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

#[derive(Debug)]
pub struct Ident<'a> {
    pub raw: &'a str,
}

impl<'a> DaedalusDisplay for Ident<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl<'a> Ident<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let raw = lexer.eat_token(Token::Ident)?;
        Ok(Self { raw })
    }
}
