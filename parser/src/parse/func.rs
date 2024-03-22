use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::Block;

#[derive(Debug)]
pub struct FunctionDefinition<'a> {
    pub ident: &'a str,
    pub ty: &'a str,
    pub block: Block<'a>,
}

impl<'a> DaedalusDisplay for FunctionDefinition<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "func {} {}() ", self.ty, self.ident)?;
        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl<'a> FunctionDefinition<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Func)?;

        let ty = lexer.eat_ident()?;
        let ident = lexer.eat_ident()?;

        lexer.eat_token(Token::OpenParen)?;
        lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(lexer)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self { ident, ty, block })
    }
}
