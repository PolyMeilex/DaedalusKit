use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    lex::{DaedalusLexer, Token},
    ParseError,
};
use std::fmt::Write;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub ident: &'a str,
}

impl<'a> DaedalusDisplay for FunctionCall<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        writeln!(f, "{}();", self.ident)?;
        Ok(())
    }
}

impl<'a> FunctionCall<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let ident = lexer.eat_ident()?;

        Self::parse_paren(lexer)?;

        lexer.eat_token(Token::Semi)?;

        Ok(Self { ident })
    }

    fn parse_paren(lexer: &mut DaedalusLexer<'a>) -> Result<(), ParseError> {
        lexer.eat_token(Token::OpenParen)?;

        let mut nest = 1;
        loop {
            match lexer.eat_one()? {
                Token::OpenParen => {
                    nest += 1;
                }
                Token::CloseParen => {
                    nest -= 1;

                    if nest == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
