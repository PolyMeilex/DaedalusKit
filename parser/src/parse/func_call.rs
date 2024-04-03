use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use daedalus_lexer::{DaedalusLexer, Token};
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
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        let ident = Ident::parse(lexer)?;

        let args = Self::parse_paren(lexer)?;

        Ok(Self { ident, args })
    }

    fn parse_paren(lexer: &mut DaedalusLexer) -> Result<Vec<Expr>, ParseError> {
        let mut out = Vec::new();

        lexer.eat_token(Token::OpenParen)?;

        if lexer.peek()? != Token::CloseParen {
            let expr = Expr::parse(lexer)?;
            out.push(expr);
        }

        loop {
            match lexer.peek()? {
                Token::CloseParen => {
                    lexer.eat_token(Token::CloseParen)?;
                    break;
                }
                Token::Comma => {
                    lexer.eat_token(Token::Comma)?;
                    let expr = Expr::parse(lexer)?;
                    out.push(expr);
                }
                got => {
                    lexer.eat_any()?;
                    return Err(ParseError::unexpeced_token(got, lexer.span()));
                }
            }
        }

        Ok(out)
    }
}
