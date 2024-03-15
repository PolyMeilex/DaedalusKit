use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    lex::{DaedalusLexer, Token},
    ParseError,
};
use std::fmt::Write;

use super::{FunctionCall, IfStatement, VarDeclaration};

#[derive(Debug)]
pub enum BlockItem<'a> {
    Var(VarDeclaration<'a>),
    If(IfStatement<'a>),
    FnCall(FunctionCall<'a>),
}

#[derive(Debug)]
pub struct Block<'a> {
    pub items: Vec<BlockItem<'a>>,
}

impl<'a> DaedalusDisplay for Block<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "{{")?;

        f.push_indent();
        for item in self.items.iter() {
            match item {
                BlockItem::Var(var) => {
                    var.fmt(f)?;
                }
                BlockItem::If(i) => {
                    i.fmt(f)?;
                }
                BlockItem::FnCall(call) => {
                    call.fmt(f)?;
                }
            }
        }
        f.pop_indent();

        f.write_indent()?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl<'a> Block<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
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
                    items.push(BlockItem::Var(VarDeclaration::parse(lexer)?));
                    continue;
                }
                Token::If => {
                    items.push(BlockItem::If(IfStatement::parse(lexer)?));
                    continue;
                }
                Token::Ident => {
                    let mut tmp = lexer.clone();
                    tmp.eat_one()?;
                    if tmp.peek()? == Token::OpenParen {
                        items.push(BlockItem::FnCall(FunctionCall::parse(lexer)?));
                        continue;
                    }
                }
                _ => {}
            }

            lexer.eat_one()?;
        }

        lexer.eat_token(Token::CloseBrace)?;

        Ok(Self { items })
    }
}
