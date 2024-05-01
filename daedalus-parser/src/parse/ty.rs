use daedalus_lexer::{Token, TokenError};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    DaedalusParser, ParseError,
};

#[derive(Debug)]
pub struct Ty {
    pub raw: String,
}

impl DaedalusDisplay for Ty {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl Ty {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        match ctx.lexer.peek()? {
            Token::Ident => {
                let raw = ctx.lexer.eat_token(Token::Ident)?;
                Ok(Self {
                    raw: raw.to_string(),
                })
            }
            Token::Func => {
                let raw = ctx.lexer.eat_token(Token::Func)?;
                Ok(Self {
                    raw: raw.to_string(),
                })
            }
            Token::Instance => {
                let raw = ctx.lexer.eat_token(Token::Instance)?;
                Ok(Self {
                    raw: raw.to_string(),
                })
            }
            got => {
                ctx.lexer.eat_any()?;
                Err(TokenError::unexpeced_token(got, ctx.lexer.span()).into())
            }
        }
    }
}
