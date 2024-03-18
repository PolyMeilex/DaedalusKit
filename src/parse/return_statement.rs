use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    lex::{DaedalusLexer, Token},
    ParseError,
};
use std::fmt::Write;

#[derive(Debug)]
pub struct ReturnStatement<'a> {
    pub args: &'a str,
}

impl<'a> DaedalusDisplay for ReturnStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        writeln!(f, "return {};", self.args)?;
        Ok(())
    }
}

impl<'a> ReturnStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Return)?;

        let args = Self::parse_args(lexer)?;

        Ok(Self { args })
    }

    fn parse_args(lexer: &mut DaedalusLexer<'a>) -> Result<&'a str, ParseError> {
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
