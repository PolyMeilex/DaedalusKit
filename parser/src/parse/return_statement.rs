use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::Expr;

#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub expr: Expr<'a>,
}

impl<'a> DaedalusDisplay for ReturnStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "return ")?;
        self.expr.fmt(f)?;
        writeln!(f, ";")?;
        Ok(())
    }
}

impl<'a> ReturnStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Return)?;

        let expr = Expr::parse(lexer)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self { expr })
    }
}
