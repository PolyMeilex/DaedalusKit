use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::Block;

#[derive(Debug)]
pub struct Instance<'a> {
    pub ident: &'a str,
    pub parent: &'a str,
    pub block: Block<'a>,
}

impl<'a> DaedalusDisplay for Instance<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "instance {}({}) ", self.ident, self.parent)?;
        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl<'a> Instance<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Instance)?;

        let ident = lexer.eat_token(Token::Ident)?;

        lexer.eat_token(Token::OpenParen)?;

        let parent = lexer.eat_token(Token::Ident)?;

        lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(lexer)?;

        lexer.eat_token(Token::Semi)?;

        Ok(Self {
            ident,
            parent,
            block,
        })
    }
}
