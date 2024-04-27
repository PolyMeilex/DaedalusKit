use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use daedalus_lexer::{DaedalusLexer, Token};
use logos::Span;
use std::fmt::Write;

use super::{Expr, Ident, Ty};

#[derive(Debug)]
pub enum ConstKind {
    Value { init: Expr },
    Array { size_init: Expr, init: Vec<Expr> },
}

#[derive(Debug)]
pub struct Const {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: ConstKind,
    pub span: Span,
}

impl DaedalusDisplay for Const {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "const ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;

        match &self.kind {
            ConstKind::Value { init } => {
                write!(f, " = ")?;
                init.fmt(f)?;
            }
            ConstKind::Array { size_init, init } => {
                write!(f, "[")?;
                size_init.fmt(f)?;
                write!(f, "]")?;

                write!(f, " = {{")?;

                let mut iter = init.iter().peekable();
                while let Some(expr) = iter.next() {
                    expr.fmt(f)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")?;
            }
        }

        Ok(())
    }
}

impl Const {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Const)?;
        let start = lexer.span().start;

        let ty = Ty::parse(lexer)?;
        let ident = Ident::parse(lexer)?;

        let kind = if lexer.peek()? == Token::OpenBracket {
            lexer.eat_token(Token::OpenBracket)?;
            let size_init = Expr::parse(lexer)?;
            lexer.eat_token(Token::CloseBracket)?;

            lexer.eat_token(Token::Eq)?;

            let init = {
                let mut inits = Vec::new();

                lexer.eat_token(Token::OpenBrace)?;
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

                inits
            };

            ConstKind::Array { size_init, init }
        } else {
            lexer.eat_token(Token::Eq)?;

            ConstKind::Value {
                init: Expr::parse(lexer)?,
            }
        };

        let end = lexer.span().end;

        Ok(Self {
            ident,
            ty,
            kind,
            span: start..end,
        })
    }
}
