use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Expr, Ty};

#[derive(Debug)]
pub struct Const<'a> {
    pub ident: &'a str,
    pub ty: Ty<'a>,
    pub arr: Option<Expr<'a>>,
    pub expr: Option<Expr<'a>>,
}

impl<'a> DaedalusDisplay for Const<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "const ")?;
        self.ty.fmt(f)?;
        write!(f, " {}", self.ident)?;

        if let Some(arr) = self.arr.as_ref() {
            write!(f, "[")?;
            arr.fmt(f)?;
            write!(f, "]")?;
        }

        if let Some(expr) = self.expr.as_ref() {
            write!(f, " = ")?;
            expr.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> Const<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Const)?;

        let ty = Ty::parse(lexer)?;
        let ident = lexer.eat_token(Token::Ident)?;

        let arr = if lexer.peek()? == Token::OpenBracket {
            lexer.eat_token(Token::OpenBracket)?;
            let index = Expr::parse(lexer)?;
            lexer.eat_token(Token::CloseBracket)?;
            Some(index)
        } else {
            None
        };

        let expr = if lexer.peek()? == Token::Eq {
            lexer.eat_token(Token::Eq)?;
            Some(Expr::parse(lexer)?)
        } else {
            None
        };

        Ok(Self {
            ident,
            ty,
            arr,
            expr,
        })
    }
}
