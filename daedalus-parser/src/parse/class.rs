use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;
use logos::Span;

use super::{Ident, Var};

#[derive(Debug)]
pub struct Class {
    pub ident: Ident,
    pub fields: Vec<Var>,
    pub span: Span,
}

impl Class {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Class)?;
        let start = ctx.lexer.span().start;

        let ident = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::OpenBrace)?;

        let mut fields = Vec::new();
        loop {
            if ctx.lexer.peek()? == Token::CloseBrace {
                break;
            }

            fields.push(Var::parse(ctx)?);
            ctx.lexer.eat_token(Token::Semi)?;
        }

        ctx.lexer.eat_token(Token::CloseBrace)?;
        ctx.lexer.eat_token(Token::Semi)?;
        let end = ctx.lexer.span().end;

        Ok(Self {
            ident,
            fields,
            span: start..end,
        })
    }
}
