use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
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
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        match lexer.peek()? {
            Token::Ident => {
                let raw = lexer.eat_token(Token::Ident)?;
                Ok(Self {
                    raw: raw.to_string(),
                })
            }
            Token::Func => {
                let raw = lexer.eat_token(Token::Func)?;
                Ok(Self {
                    raw: raw.to_string(),
                })
            }
            got => {
                lexer.eat_any()?;
                Err(ParseError::unexpeced_token(got, lexer.span()))
            }
        }
    }
}
