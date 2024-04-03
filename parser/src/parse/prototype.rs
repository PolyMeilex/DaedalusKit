use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use daedalus_lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Block, Ident};

#[derive(Debug)]
pub struct Prototype {
    pub ident: Ident,
    pub parent: Ident,
    pub block: Block,
}

impl DaedalusDisplay for Prototype {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "prototype ")?;
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

impl Prototype {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Prototype)?;

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
