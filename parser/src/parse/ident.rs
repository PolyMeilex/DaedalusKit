use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

#[derive(Debug)]
pub struct Ident {
    pub raw: String,
}

impl DaedalusDisplay for Ident {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl Ident {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        let raw = lexer.eat_token(Token::Ident)?;
        Ok(Self {
            raw: raw.to_string(),
        })
    }
}
