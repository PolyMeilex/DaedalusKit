use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Block, Ident};

#[derive(Debug)]
pub struct Instance {
    pub ident: Ident,
    pub parent: Ident,
    pub block: Block,
}

impl DaedalusDisplay for Instance {
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

impl Instance {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
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
