use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

#[derive(Debug)]
pub struct Field<'a> {
    pub ty: &'a str,
    pub ident: &'a str,
    pub arr: Option<&'a str>,
}

#[derive(Debug)]
pub struct Class<'a> {
    pub ident: &'a str,
    pub parent: &'a str,
    pub fields: Vec<Field<'a>>,
}

impl<'a> DaedalusDisplay for Class<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "class {}({}) {{", self.ident, self.parent)?;

        f.push_indent();
        for Field { ident, ty, arr } in self.fields.iter() {
            f.write_indent()?;
            if let Some(arr) = arr {
                writeln!(f, "var {ty} {ident}[{arr}];")?;
            } else {
                writeln!(f, "var {ty} {ident};")?;
            }
        }
        f.pop_indent();

        writeln!(f, "}};")?;
        writeln!(f)?;

        Ok(())
    }
}

impl<'a> Class<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Instance)?;

        let ident = lexer.eat_token(Token::Ident)?;

        lexer.eat_token(Token::OpenParen)?;

        let parent = lexer.eat_token(Token::Ident)?;

        lexer.eat_token(Token::CloseParen)?;

        lexer.eat_token(Token::OpenBrace)?;

        let mut fields = Vec::new();
        loop {
            if lexer.peek()? == Token::CloseBrace {
                break;
            }

            fields.push(Self::parse_field(lexer)?);
        }

        lexer.eat_token(Token::CloseBrace)?;
        lexer.eat_token(Token::Semi)?;

        Ok(Self {
            ident,
            parent,
            fields,
        })
    }

    fn parse_field(lexer: &mut DaedalusLexer<'a>) -> Result<Field<'a>, ParseError> {
        lexer.eat_token(Token::Var)?;
        let ty = lexer.eat_token(Token::Ident)?;
        let ident = lexer.eat_token(Token::Ident)?;

        let arr = if lexer.peek()? == Token::OpenBracket {
            lexer.eat_token(Token::OpenBracket)?;

            // TODO: This should be a proper expression parser
            let start = lexer.span().end;
            loop {
                if lexer.peek()? == Token::CloseBracket {
                    break;
                }
            }
            let end = lexer.span().start;
            let str = lexer.inner().source().get(start..end).unwrap();

            lexer.eat_token(Token::CloseBracket)?;
            Some(str)
        } else {
            None
        };

        lexer.eat_token(Token::Semi)?;

        Ok(Field { ty, ident, arr })
    }
}
