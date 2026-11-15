use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;
use logos::Span;

use super::Expr;

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expr>,
    pub span: Span,
}

impl ReturnStatement {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Return)?;
        let start = ctx.lexer.span().start;

        let expr = if ctx.lexer.peek()? != Token::Semi {
            let expr = Expr::parse(ctx)?;
            Some(expr)
        } else {
            None
        };

        ctx.lexer.eat_token(Token::Semi)?;

        Ok(Self {
            expr,
            span: start..ctx.lexer.span().end,
        })
    }
}
