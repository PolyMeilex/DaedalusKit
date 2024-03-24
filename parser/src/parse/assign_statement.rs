use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

#[derive(Debug)]
pub struct AssignStatement<'a> {
    pub a: &'a str,
    pub b: &'a str,
}

impl<'a> DaedalusDisplay for AssignStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        writeln!(f, "{} = {};", self.a, self.b)?;
        Ok(())
    }
}

impl<'a> AssignStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let a = lexer.eat_token(Token::Ident)?;
        lexer.eat_token(Token::Eq)?;
        let b = Self::parse_right(lexer)?;

        Ok(Self { a, b })
    }

    fn parse_right(lexer: &mut DaedalusLexer<'a>) -> Result<&'a str, ParseError> {
        let start = lexer.span().end;

        loop {
            if lexer.eat_one()? == Token::Semi {
                break;
            }
        }

        let end = lexer.span().start;

        let str = lexer.inner().source().get(start..end).unwrap();

        Ok(str)
    }
}
