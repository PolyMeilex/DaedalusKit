use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Expr, Ident, Ty};

#[derive(Debug)]
pub enum ConstKind {
    Value {
        init: Option<Expr>,
    },
    Array {
        size_init: Expr,
        init: Option<Vec<Expr>>,
    },
}

#[derive(Debug)]
pub struct Const {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: ConstKind,
}

impl DaedalusDisplay for Const {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "const ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;

        match &self.kind {
            ConstKind::Value { init: Some(init) } => {
                write!(f, " = ")?;
                init.fmt(f)?;
            }
            ConstKind::Array { size_init, init } => {
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

impl Const {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Const)?;

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

            ConstKind::Array { size_init, init }
        } else {
            let init = if lexer.peek()? == Token::Eq {
                lexer.eat_token(Token::Eq)?;
                Some(Expr::parse(lexer)?)
            } else {
                None
            };

            ConstKind::Value { init }
        };

        Ok(Self { ident, ty, kind })
    }
}
