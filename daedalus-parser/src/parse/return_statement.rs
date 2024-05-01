use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;

use super::Expr;

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expr>,
}

impl ReturnStatement {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Return)?;

        let expr = if ctx.lexer.peek()? != Token::Semi {
            let expr = Expr::parse(ctx)?;
            Some(expr)
        } else {
            None
        };

        ctx.lexer.eat_token(Token::Semi)?;

        Ok(Self { expr })
    }
}
