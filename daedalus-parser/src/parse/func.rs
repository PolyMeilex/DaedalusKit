use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;
use logos::Span;

use super::{Block, Ident, Ty, Var};

#[derive(Debug)]
pub struct FunctionDefinition {
    pub ident: Ident,
    pub ty: Ty,
    pub args: Vec<Var>,
    pub block: Block,
    pub span: Span,
}

impl FunctionDefinition {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Func)?;
        let start = ctx.lexer.span().start;

        let ty = Ty::parse(ctx)?;
        let ident = Ident::parse(ctx)?;

        ctx.lexer.eat_token(Token::OpenParen)?;

        let mut args = Vec::new();
        loop {
            if ctx.lexer.peek()? == Token::CloseParen {
                break;
            }
            args.push(Var::parse(ctx)?);

            if ctx.lexer.peek()? == Token::Comma {
                ctx.lexer.eat_token(Token::Comma)?;
            } else {
                break;
            }
        }

        ctx.lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(ctx)?;
        ctx.lexer.eat_token(Token::Semi)?;
        let end = ctx.lexer.span().end;

        Ok(Self {
            ident,
            ty,
            args,
            block,
            span: start..end,
        })
    }
}
