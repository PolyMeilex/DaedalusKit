use daedalus_lexer::Token;
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    DaedalusParser, ParseError,
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
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let raw = ctx.lexer.eat_token(Token::Ident)?;
        Ok(Self {
            raw: raw.to_string(),
        })
    }
}
