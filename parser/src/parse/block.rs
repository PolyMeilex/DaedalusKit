use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{AssignStatement, FunctionCall, IfStatement, ReturnStatement, Var};

#[derive(Debug)]
pub enum BlockItem<'a> {
    Var(Var<'a>),
    If(IfStatement<'a>),
    FnCall(FunctionCall<'a>),
    Return(ReturnStatement<'a>),
    Assign(AssignStatement<'a>),
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
                    f.write_indent()?;
                    call.fmt(f)?;
                    writeln!(f, ";")?;
                }
                BlockItem::Return(ret) => {
                    ret.fmt(f)?;
                }
                BlockItem::Assign(assign) => {
                    assign.fmt(f)?;
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
                    items.push(BlockItem::Var(Var::parse(lexer)?));
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
                    let mut tmp = lexer.clone();
                    tmp.eat_any()?;

                    match tmp.peek()? {
                        Token::OpenParen => {
                            items.push(BlockItem::FnCall(FunctionCall::parse(lexer)?));
                            lexer.eat_token(Token::Semi)?;
                            continue;
                        }
                        Token::Eq => {
                            items.push(BlockItem::Assign(AssignStatement::parse(lexer)?));
                            continue;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            lexer.eat_any()?;
        }

        lexer.eat_token(Token::CloseBrace)?;

        Ok(Self { items })
    }
}
