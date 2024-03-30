use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Block, Ty, Var};

#[derive(Debug)]
pub struct FunctionDefinition<'a> {
    pub ident: &'a str,
    pub ty: Ty<'a>,
    pub args: Vec<Var<'a>>,
    pub block: Block<'a>,
}

impl<'a> DaedalusDisplay for FunctionDefinition<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "func ")?;
        self.ty.fmt(f)?;
        write!(f, " {}(", self.ident)?;

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

impl<'a> FunctionDefinition<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Func)?;

        let ty = Ty::parse(lexer)?;
        let ident = lexer.eat_token(Token::Ident)?;

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

        Ok(Self {
            ident,
            ty,
            args,
            block,
        })
    }
}
