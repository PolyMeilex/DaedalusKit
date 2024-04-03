use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use logos::Span;
use std::fmt::Write;

use super::{Block, Ident, Ty, Var};

#[derive(Debug)]
pub struct FunctionDefinition {
    pub ident: Ident,
    pub ty: Ty,
    pub args: Vec<Var>,
    pub block: Block,
    pub span: Span,
}

impl DaedalusDisplay for FunctionDefinition {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "func ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;

        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        write!(f, ") ")?;
        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl FunctionDefinition {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Func)?;
        let start = lexer.span().start;

        let ty = Ty::parse(lexer)?;
        let ident = Ident::parse(lexer)?;

        lexer.eat_token(Token::OpenParen)?;

        let mut args = Vec::new();
        loop {
            if lexer.peek()? == Token::CloseParen {
                break;
            }
            args.push(Var::parse(lexer)?);

            if lexer.peek()? == Token::Comma {
                lexer.eat_token(Token::Comma)?;
            } else {
                break;
            }
        }

        lexer.eat_token(Token::CloseParen)?;

        let block = Block::parse(lexer)?;
        lexer.eat_token(Token::Semi)?;
        let end = lexer.span().end;

        Ok(Self {
            ident,
            ty,
            args,
            block,
            span: start..end,
        })
    }
}
