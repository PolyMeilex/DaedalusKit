use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    DaedalusParser, ParseError,
};
use daedalus_lexer::{Token, TokenError};
use std::fmt::Write;

use super::{Expr, Ident};

#[derive(Debug)]
pub struct FunctionCall {
    pub ident: Ident,
    pub args: Vec<Expr>,
}

impl DaedalusDisplay for FunctionCall {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        self.ident.fmt(f)?;
        write!(f, "(")?;
        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl FunctionCall {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let ident = Ident::parse(ctx)?;

        let args = Self::parse_paren(ctx)?;

        Ok(Self { ident, args })
    }

    fn parse_paren(ctx: &mut DaedalusParser) -> Result<Vec<Expr>, ParseError> {
        let mut out = Vec::new();

        ctx.lexer.eat_token(Token::OpenParen)?;

        if ctx.lexer.peek()? != Token::CloseParen {
            let expr = Expr::parse(ctx)?;
            out.push(expr);
        }

        loop {
            match ctx.lexer.peek()? {
                Token::CloseParen => {
                    ctx.lexer.eat_token(Token::CloseParen)?;
                    break;
                }
                Token::Comma => {
                    ctx.lexer.eat_token(Token::Comma)?;
                    let expr = Expr::parse(ctx)?;
                    out.push(expr);
                }
                got => {
                    ctx.lexer.eat_any()?;
                    return Err(TokenError::unexpeced_token(got, ctx.lexer.span()).into());
                }
            }
        }

        Ok(out)
    }
}
