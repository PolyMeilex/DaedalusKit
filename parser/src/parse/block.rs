use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use daedalus_lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Expr, IfStatement, ReturnStatement, Var};

#[derive(Debug)]
pub enum BlockItem {
    Var(Var),
    If(IfStatement),
    Return(ReturnStatement),
    Expr(Expr),
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

impl DaedalusDisplay for Block {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        if self.items.is_empty() {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;

        f.push_indent();
        for item in self.items.iter() {
            match item {
                BlockItem::Var(var) => {
                    var.fmt(f)?;
                    writeln!(f, ";")?;
                }
                BlockItem::If(i) => {
                    i.fmt(f)?;
                }
                BlockItem::Return(ret) => {
                    ret.fmt(f)?;
                }
                BlockItem::Expr(expr) => {
                    f.write_indent()?;
                    expr.fmt(f)?;
                    writeln!(f, ";")?;
                }
            }
        }
        f.pop_indent();

        f.write_indent()?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl Block {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::OpenBrace)?;

        let mut items = Vec::new();
        let mut nest = 1;
        loop {
            match lexer.peek()? {
                Token::OpenBrace => {
                    nest += 1;
                }
                Token::CloseBrace => {
                    nest -= 1;

                    if nest == 0 {
                        break;
                    }
                }
                Token::Var => {
                    items.push(BlockItem::Var(Var::parse(lexer)?));
                    lexer.eat_token(Token::Semi)?;
                    continue;
                }
                Token::If => {
                    items.push(BlockItem::If(IfStatement::parse(lexer)?));
                    continue;
                }
                Token::Return => {
                    items.push(BlockItem::Return(ReturnStatement::parse(lexer)?));
                    continue;
                }
                Token::Ident => {
                    items.push(BlockItem::Expr(Expr::parse(lexer)?));
                    lexer.eat_token(Token::Semi)?;
                }
                got => {
                    lexer.eat_any()?;
                    return Err(ParseError::unexpeced_token(got, lexer.span()));
                }
            }
        }

        lexer.eat_token(Token::CloseBrace)?;

        Ok(Self { items })
    }
}
