use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    lex::{DaedalusLexer, Token},
    ParseError,
};
use std::fmt::Write;

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub ident: &'a str,
    pub trailing_comment: Option<&'a str>,
}

impl<'a> DaedalusDisplay for FunctionCall<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "{}();", self.ident)?;
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

        Self::parse_paren(lexer)?;

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
            trailing_comment,
        })
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
