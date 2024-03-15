use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    lex::{DaedalusLexer, Token},
    ParseError,
};
use std::fmt::Write;

use super::Block;

#[derive(Debug)]
pub struct IfStatement<'a> {
    pub has_else: bool,
    pub has_if: bool,
    pub block: Block<'a>,
    pub next: Option<Box<IfStatement<'a>>>,
}

impl<'a> DaedalusDisplay for IfStatement<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        if self.has_else && self.has_if {
            write!(f, " else if() ")?;
        } else if self.has_if {
            f.write_indent()?;
            write!(f, "if() ")?;
        } else if self.has_else {
            write!(f, " else ")?;
        }

        self.block.fmt(f)?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        } else {
            writeln!(f, ";")?;
        }

        Ok(())
    }
}

impl<'a> IfStatement<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let (has_else, has_if) = if lexer.peek()? == Token::Else {
            lexer.eat_token(Token::Else)?;

            let has_if = lexer.peek()? == Token::If;

            (true, has_if)
        } else {
            (false, true)
        };

        if has_if {
            lexer.eat_token(Token::If)?;
            Self::parse_paren(lexer)?;
        }

        let block = Block::parse(lexer)?;

        let mut next = None;

        if lexer.peek()? == Token::Else {
            let stmt = IfStatement::parse(lexer)?;
            next = Some(Box::new(stmt));
        } else {
            lexer.eat_token(Token::Semi)?;
        }

        Ok(Self {
            block,
            has_else,
            has_if,
            next,
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
