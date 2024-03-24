use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::Expr;

#[derive(Debug)]
pub struct AssignStatement<'a> {
    pub a: &'a str,
    pub expr: Expr<'a>,
}

impl<'a> DaedalusDisplay for AssignStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "{} = ", self.a)?;
        self.expr.fmt(f)?;
        writeln!(f, ";")?;
        Ok(())
    }
}

impl<'a> AssignStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let a = lexer.eat_token(Token::Ident)?;
        lexer.eat_token(Token::Eq)?;
        let expr = Expr::parse(lexer)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self { a, expr })
    }
}
