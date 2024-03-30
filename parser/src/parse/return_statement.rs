use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::Expr;

#[derive(Debug)]
pub struct ReturnStatement {
    pub expr: Option<Expr>,
}

impl DaedalusDisplay for ReturnStatement {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "return")?;
        if let Some(expr) = self.expr.as_ref() {
            write!(f, " ")?;
            expr.fmt(f)?;
        }
        writeln!(f, ";")?;
        Ok(())
    }
}

impl ReturnStatement {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Return)?;

        let expr = if lexer.peek()? != Token::Semi {
            let expr = Expr::parse(lexer)?;
            Some(expr)
        } else {
            None
        };

        lexer.eat_token(Token::Semi)?;

        Ok(Self { expr })
    }
}
