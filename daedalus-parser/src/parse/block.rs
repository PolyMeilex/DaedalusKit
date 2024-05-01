use crate::{DaedalusParser, ParseError};
use daedalus_lexer::{Token, TokenError};

use super::{Expr, IfStatement, ReturnStatement, Var};

#[derive(Debug)]
pub enum BlockItem {
    Var(Var),
    If(IfStatement),
    Return(ReturnStatement),
    Expr(Expr),
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

impl Block {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::OpenBrace)?;

        let mut items = Vec::new();
        let mut nest = 1;
        loop {
            match ctx.lexer.peek()? {
                Token::OpenBrace => {
                    nest += 1;
                }
                Token::CloseBrace => {
                    nest -= 1;

                    if nest == 0 {
                        break;
                    }
                }
                Token::Var => {
                    items.push(BlockItem::Var(Var::parse(ctx)?));
                    ctx.lexer.eat_token(Token::Semi)?;
                    continue;
                }
                Token::If => {
                    items.push(BlockItem::If(IfStatement::parse(ctx)?));
                    continue;
                }
                Token::Return => {
                    items.push(BlockItem::Return(ReturnStatement::parse(ctx)?));
                    continue;
                }
                Token::Ident => {
                    items.push(BlockItem::Expr(Expr::parse(ctx)?));
                    ctx.lexer.eat_token(Token::Semi)?;
                }
                got => {
                    ctx.lexer.eat_any()?;
                    return Err(TokenError::unexpeced_token(got, ctx.lexer.span()).into());
                }
            }
        }

        ctx.lexer.eat_token(Token::CloseBrace)?;

        Ok(Self { items })
    }
}
