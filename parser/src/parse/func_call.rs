use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};
use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub ident: &'a str,
    pub args: &'a str,
    pub trailing_comment: Option<&'a str>,
}

impl<'a> DaedalusDisplay for FunctionCall<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "{}({});", self.ident, self.args)?;
        if let Some(comment) = self.trailing_comment {
            write!(f, " {}", comment)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl<'a> FunctionCall<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let ident = lexer.eat_ident()?;

        let args = Self::parse_paren(lexer)?;

        lexer.eat_token(Token::Semi)?;

        let trailing_comment = if lexer.peek_with_comments().ok() == Some(Token::LineComment) {
            lexer.eat_one_raw()?;
            let src = lexer.inner().slice();
            Some(src)
        } else {
            None
        };

        Ok(Self {
            ident,
            args,
            trailing_comment,
        })
    }

    fn parse_paren(lexer: &mut DaedalusLexer<'a>) -> Result<&'a str, ParseError> {
        lexer.eat_token(Token::OpenParen)?;

        let start = lexer.span().end;

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

        let end = lexer.span().start;

        let str = lexer.inner().source().get(start..end).unwrap();

        Ok(str)
    }
}
