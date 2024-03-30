use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Block, Ident};

#[derive(Debug)]
pub struct Instance<'a> {
    pub ident: Ident<'a>,
    pub parent: Ident<'a>,
    pub block: Block<'a>,
}

impl<'a> DaedalusDisplay for Instance<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "instance ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;
        self.parent.fmt(f)?;
        write!(f, ") ")?;

        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl<'a> Instance<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Instance)?;

        let ident = Ident::parse(lexer)?;

        lexer.eat_token(Token::OpenParen)?;

        let parent = Ident::parse(lexer)?;

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
