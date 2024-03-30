use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Ident, Ty, Var};

// TODO: Are extern functions even part of the language?
#[derive(Debug)]
pub struct ExternFunctionDefinition {
    pub ident: Ident,
    pub ty: Ty,
    pub args: Vec<Var>,
}

impl DaedalusDisplay for ExternFunctionDefinition {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "extern func ")?;
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

        writeln!(f, ");")?;
        Ok(())
    }
}

impl ExternFunctionDefinition {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Extern)?;
        lexer.eat_token(Token::Func)?;

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

        if lexer.peek()? == Token::Semi {
            lexer.eat_token(Token::Semi)?;
        }

        Ok(Self { ident, ty, args })
    }
}
