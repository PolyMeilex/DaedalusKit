use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    DaedalusParser, ParseError,
};
use daedalus_lexer::Token;
use logos::Span;
use std::fmt::Write;

use super::{Expr, Ident, Ty};

#[derive(Debug)]
pub enum VarKind {
    Value {
        init: Option<Expr>,
    },
    Array {
        size_init: Expr,
        init: Option<Vec<Expr>>,
    },
}

#[derive(Debug)]
pub struct Var {
    pub ident: Ident,
    pub ty: Ty,
    pub kind: VarKind,
    pub span: Span,
}

impl DaedalusDisplay for Var {
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
            _ => {}
        }

        Ok(())
    }
}

impl Var {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Var)?;
        let start = ctx.lexer.span().start;

        let ty = Ty::parse(ctx)?;
        let ident = Ident::parse(ctx)?;

        let kind = if ctx.lexer.peek()? == Token::OpenBracket {
            ctx.lexer.eat_token(Token::OpenBracket)?;
            let size_init = Expr::parse(ctx)?;
            ctx.lexer.eat_token(Token::CloseBracket)?;

            let init = if ctx.lexer.peek()? == Token::Eq {
                ctx.lexer.eat_token(Token::Eq)?;
                ctx.lexer.eat_token(Token::OpenBrace)?;

                let mut inits = Vec::new();

                loop {
                    let init = Expr::parse(ctx)?;
                    inits.push(init);

                    if ctx.lexer.peek()? == Token::CloseBrace {
                        ctx.lexer.eat_token(Token::CloseBrace)?;
                        break;
                    } else {
                        // This is not the last element comma is mandatory
                        ctx.lexer.eat_token(Token::Comma)?;
                    }
                }

                Some(inits)
            } else {
                None
            };

            VarKind::Array { size_init, init }
        } else {
            let init = if ctx.lexer.peek()? == Token::Eq {
                ctx.lexer.eat_token(Token::Eq)?;
                Some(Expr::parse(ctx)?)
            } else {
                None
            };

            VarKind::Value { init }
        };
        let end = ctx.lexer.span().end;

        Ok(Self {
            ident,
            ty,
            kind,
            span: start..end,
        })
    }
}
