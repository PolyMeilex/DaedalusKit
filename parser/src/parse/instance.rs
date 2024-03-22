use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

#[derive(Debug)]
pub struct Instance<'a> {
    pub ident: &'a str,
    pub parent: &'a str,
    pub fields: Vec<(&'a str, &'a str)>,
}

impl<'a> DaedalusDisplay for Instance<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "instance {}({}) {{", self.ident, self.parent)?;

        f.push_indent();
        for (ident, value) in self.fields.iter() {
            f.write_indent()?;
            writeln!(f, "{ident} = {value};")?;
        }
        f.pop_indent();

        writeln!(f, "}};\n")?;
        Ok(())
    }
}

impl<'a> Instance<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Instance)?;

        let ident = lexer.eat_ident()?;

        lexer.eat_token(Token::OpenParen)?;

        let parent = lexer.eat_ident()?;

        lexer.eat_token(Token::CloseParen)?;
        lexer.eat_token(Token::OpenBrace)?;

        let mut fields = Vec::new();
        loop {
            if lexer.peek()? == Token::CloseBrace {
                break;
            }

            let (ident, value) = Self::parse_field(lexer)?;
            fields.push((ident, value));
        }

        lexer.eat_token(Token::CloseBrace)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self {
            ident,
            parent,
            fields,
        })
    }

    fn parse_field(lexer: &mut DaedalusLexer<'a>) -> Result<(&'a str, &'a str), ParseError> {
        let ident = lexer.eat_ident()?;

        lexer.eat_token(Token::Eq)?;

        let peek = lexer.peek()?;

        let (Token::Ident | Token::Integer | Token::String) = peek else {
            return Err(ParseError::unexpeced_token(peek, lexer.span()));
        };

        lexer.inner().next().unwrap().unwrap();
        let value = lexer.inner().slice();

        lexer.eat_token(Token::Semi)?;

        Ok((ident, value))
    }
}
