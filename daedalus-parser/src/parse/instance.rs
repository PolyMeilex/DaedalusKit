use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;
use logos::Span;

use super::{Block, Ident};

#[derive(Debug)]
pub struct Instance {
    pub ident: Ident,
    pub parent: Ident,
    pub block: Block,
    pub span: Span,
}

impl Instance {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Instance)?;
        let start = ctx.lexer.span().start;

        let ident = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::OpenParen)?;

        let parent = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(ctx)?;

        ctx.lexer.eat_token(Token::Semi)?;
        let end = ctx.lexer.span().end;

        Ok(Self {
            ident,
            parent,
            block,
            span: start..end,
        })
    }
}
