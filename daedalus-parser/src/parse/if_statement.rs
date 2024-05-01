use crate::{DaedalusParser, ParseError};
use daedalus_lexer::Token;

use super::{Block, Expr};

#[derive(Debug)]
pub struct IfStatement {
    pub has_else: bool,
    pub has_if: bool,
    pub has_semi: bool,
    pub block: Block,
    pub condition: Option<Expr>,
    pub next: Option<Box<IfStatement>>,
}

impl IfStatement {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let (has_else, has_if) = if ctx.lexer.peek()? == Token::Else {
            ctx.lexer.eat_token(Token::Else)?;

            let has_if = ctx.lexer.peek()? == Token::If;

            (true, has_if)
        } else {
            (false, true)
        };

        let condition = if has_if {
            ctx.lexer.eat_token(Token::If)?;
            Some(Expr::parse(ctx)?)
        } else {
            None
        };

        let block = Block::parse(ctx)?;

        let mut next = None;

        let has_semi = if ctx.lexer.peek()? == Token::Else {
            let stmt = IfStatement::parse(ctx)?;
            next = Some(Box::new(stmt));
            false
        } else if ctx.lexer.peek()? == Token::Semi {
            ctx.lexer.eat_token(Token::Semi)?;
            true
        } else {
            false
        };

        Ok(Self {
            block,
            has_else,
            has_if,
            has_semi,
            condition,
            next,
        })
    }
}
