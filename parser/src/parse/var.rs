use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

#[derive(Debug)]
pub struct Var<'a> {
    pub ident: &'a str,
    pub ty: &'a str,
}

impl<'a> DaedalusDisplay for Var<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        writeln!(f, "var {} {};", self.ty, self.ident)?;

        Ok(())
    }
}

impl<'a> Var<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Var)?;

        let ty = lexer.eat_token(Token::Ident)?;
        let ident = lexer.eat_token(Token::Ident)?;

        lexer.eat_token(Token::Semi)?;

        Ok(Self { ident, ty })
    }
}
