use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Expr, Ty};

#[derive(Debug)]
pub struct Var<'a> {
    pub ident: &'a str,
    pub ty: Ty<'a>,
    pub arr: Option<Expr<'a>>,
}

impl<'a> DaedalusDisplay for Var<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "var ")?;
        self.ty.fmt(f)?;
        write!(f, " {}", self.ident)?;

        if let Some(arr) = self.arr.as_ref() {
            write!(f, "[")?;
            arr.fmt(f)?;
            write!(f, "]")?;
        }

        writeln!(f, ";")?;

        Ok(())
    }
}

impl<'a> Var<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Var)?;

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

        lexer.eat_token(Token::Semi)?;

        Ok(Self { ident, ty, arr })
    }
}
