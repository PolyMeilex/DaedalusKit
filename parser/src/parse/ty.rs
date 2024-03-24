use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

#[derive(Debug)]
pub struct Ty<'a> {
    pub raw: &'a str,
}

impl<'a> DaedalusDisplay for Ty<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl<'a> Ty<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        match lexer.peek()? {
            Token::Ident => {
                let raw = lexer.eat_token(Token::Ident)?;
                Ok(Self { raw })
            }
            Token::Func => {
                let raw = lexer.eat_token(Token::Func)?;
                Ok(Self { raw })
            }
            got => {
                lexer.eat_any()?;
                Err(ParseError::unexpeced_token(got, lexer.span()))
            }
        }
    }
}
