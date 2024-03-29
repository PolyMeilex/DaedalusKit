use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Block, Expr};

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub has_else: bool,
    pub has_if: bool,
    pub block: Block<'a>,
    pub condition: Option<Expr<'a>>,
    pub next: Option<Box<IfStatement<'a>>>,
}

impl<'a> DaedalusDisplay for IfStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        if let Some(condition) = self.condition.as_ref() {
            if self.has_else {
                write!(f, " else if(")?;
            } else {
                f.write_indent()?;
                write!(f, "if(")?;
            }

            condition.fmt(f)?;

            write!(f, ") ")?;
        } else if self.has_else {
            write!(f, " else ")?;
        }

        self.block.fmt(f)?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        } else {
            writeln!(f, ";")?;
        }

        Ok(())
    }
}

impl<'a> IfStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let (has_else, has_if) = if lexer.peek()? == Token::Else {
            lexer.eat_token(Token::Else)?;

            let has_if = lexer.peek()? == Token::If;

            (true, has_if)
        } else {
            (false, true)
        };

        let condition = if has_if {
            lexer.eat_token(Token::If)?;
            Some(Expr::parse(lexer)?)
        } else {
            None
        };

        let block = Block::parse(lexer)?;

        let mut next = None;

        if lexer.peek()? == Token::Else {
            let stmt = IfStatement::parse(lexer)?;
            next = Some(Box::new(stmt));
        } else if lexer.peek()? == Token::Semi {
            lexer.eat_token(Token::Semi)?;
        }

        Ok(Self {
            block,
            has_else,
            has_if,
            condition,
            next,
        })
    }
}
