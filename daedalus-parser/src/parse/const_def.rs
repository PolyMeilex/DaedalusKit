use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;
use logos::Span;

use super::{Expr, Ident, Ty};

#[derive(Debug)]
pub enum ConstKind {
    Value { init: Expr },
    Array { size_init: Expr, init: Vec<Expr> },
}

#[derive(Debug)]
pub struct Const {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: ConstKind,
    pub span: Span,
}

impl Const {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Const)?;
        let start = ctx.lexer.span().start;

        let ty = Ty::parse(ctx)?;
        let ident = Ident::parse(ctx)?;

        let kind = if ctx.lexer.peek()? == Token::OpenBracket {
            ctx.lexer.eat_token(Token::OpenBracket)?;
            let size_init = Expr::parse(ctx)?;
            ctx.lexer.eat_token(Token::CloseBracket)?;

            ctx.lexer.eat_token(Token::Eq)?;

            let init = {
                let mut inits = Vec::new();

                ctx.lexer.eat_token(Token::OpenBrace)?;
                loop {
                    let init = Expr::parse(ctx)?;
                    inits.push(init);

                    if ctx.lexer.peek()? == Token::CloseBrace {
                        ctx.lexer.eat_token(Token::CloseBrace)?;
                        break;
                    } else {
                        // This is not the last element comma is mandatory
                        ctx.lexer.eat_token(Token::Comma)?;
                    }
                }

                inits
            };

            ConstKind::Array { size_init, init }
        } else {
            ctx.lexer.eat_token(Token::Eq)?;

            ConstKind::Value {
                init: Expr::parse(ctx)?,
            }
        };

        let end = ctx.lexer.span().end;

        Ok(Self {
            ident,
            ty,
            kind,
            span: start..end,
        })
    }
}
