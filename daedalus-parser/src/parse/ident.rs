use daedalus_lexer::Token;

use crate::{DaedalusParser, ParseError};

#[derive(Debug)]
pub struct Ident {
    pub raw: String,
}

impl Ident {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let raw = ctx.lexer.eat_token(Token::Ident)?;
        Ok(Self {
            raw: raw.to_string(),
        })
    }
}
