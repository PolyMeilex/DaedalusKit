use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Expr, Ident, Ty};

#[derive(Debug)]
pub enum VarKind<'a> {
    Value {
        init: Option<Expr<'a>>,
    },
    Array {
        size_init: Expr<'a>,
        init: Option<Vec<Expr<'a>>>,
    },
}

#[derive(Debug)]
pub struct Var<'a> {
    pub ident: Ident<'a>,
    pub ty: Ty<'a>,
    pub kind: VarKind<'a>,
}

impl<'a> DaedalusDisplay for Var<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "var ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;

        match &self.kind {
            VarKind::Value { init: Some(init) } => {
                write!(f, " = ")?;
                init.fmt(f)?;
            }
            VarKind::Array { size_init, init } => {
                write!(f, "[")?;
                size_init.fmt(f)?;
                write!(f, "]")?;

                if let Some(init) = init {
                    write!(f, " = {{")?;
                    for expr in init {
                        expr.fmt(f)?;
                        write!(f, ", ")?;
                    }
                    write!(f, "}}")?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<'a> Var<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Var)?;

        let ty = Ty::parse(lexer)?;
        let ident = Ident::parse(lexer)?;

        let kind = if lexer.peek()? == Token::OpenBracket {
            lexer.eat_token(Token::OpenBracket)?;
            let size_init = Expr::parse(lexer)?;
            lexer.eat_token(Token::CloseBracket)?;

            let init = if lexer.peek()? == Token::Eq {
                lexer.eat_token(Token::Eq)?;
                lexer.eat_token(Token::OpenBrace)?;

                let mut inits = Vec::new();

                loop {
                    let init = Expr::parse(lexer)?;
                    inits.push(init);

                    if lexer.peek()? == Token::CloseBrace {
                        lexer.eat_token(Token::CloseBrace)?;
                        break;
                    } else {
                        // This is not the last element comma is mandatory
                        lexer.eat_token(Token::Comma)?;
                    }
                }

                Some(inits)
            } else {
                None
            };

            VarKind::Array { size_init, init }
        } else {
            let init = if lexer.peek()? == Token::Eq {
                lexer.eat_token(Token::Eq)?;
                Some(Expr::parse(lexer)?)
            } else {
                None
            };

            VarKind::Value { init }
        };

        Ok(Self { ident, ty, kind })
    }
}
