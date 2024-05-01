use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;

use super::{Block, Ident};

#[derive(Debug)]
pub struct Prototype {
    pub ident: Ident,
    pub parent: Ident,
    pub block: Block,
}

impl Prototype {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Prototype)?;

        let ident = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::OpenParen)?;

        let parent = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(ctx)?;

        ctx.lexer.eat_token(Token::Semi)?;

        Ok(Self {
            ident,
            parent,
            block,
        })
    }
}
