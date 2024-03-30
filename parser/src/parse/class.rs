use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use super::{Ident, Var};

#[derive(Debug)]
pub struct Class {
    pub ident: Ident,
    pub fields: Vec<Var>,
}

impl DaedalusDisplay for Class {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "class ")?;
        self.ident.fmt(f)?;
        writeln!(f, " {{")?;

        f.push_indent();
        for var in self.fields.iter() {
            var.fmt(f)?;
            writeln!(f, ";")?;
        }
        f.pop_indent();

        writeln!(f, "}};")?;
        writeln!(f)?;

        Ok(())
    }
}

impl Class {
    pub fn parse(lexer: &mut DaedalusLexer) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Class)?;

        let ident = Ident::parse(lexer)?;

        lexer.eat_token(Token::OpenBrace)?;

        let mut fields = Vec::new();
        loop {
            if lexer.peek()? == Token::CloseBrace {
                break;
            }

            fields.push(Var::parse(lexer)?);
            lexer.eat_token(Token::Semi)?;
        }

        lexer.eat_token(Token::CloseBrace)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self { ident, fields })
    }
}
