use daedalus_lexer::{Token, TokenError};

mod instance;
pub use instance::Instance;

mod prototype;
pub use prototype::Prototype;

mod var;
pub use var::{Var, VarKind};

mod const_def;
pub use const_def::{Const, ConstKind};

mod func;
pub use func::FunctionDefinition;

mod extern_func;
pub use extern_func::ExternFunctionDefinition;

mod func_call;
pub use func_call::FunctionCall;

mod if_statement;
pub use if_statement::IfStatement;

mod block;
pub use block::{Block, BlockItem};

mod return_statement;
pub use return_statement::ReturnStatement;

mod class;
pub use class::Class;

mod expr;
pub use expr::{AssocOp, Expr, ExprKind, Lit, LitKind, UnaryOp};

mod ty;
pub use ty::Ty;

mod ident;
pub use ident::Ident;

use crate::{DaedalusParser, ParseError};

#[derive(Debug)]
pub enum Item {
    Class(Class),
    Instance(Instance),
    Prototype(Prototype),
    Var(Var),
    Const(Const),
    Func(FunctionDefinition),
    ExternFunc(ExternFunctionDefinition),
}

pub struct File {
    pub items: Vec<Item>,
}

impl File {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let mut items = Vec::new();

        loop {
            match ctx.lexer.peek()? {
                Token::Class => {
                    items.push(Item::Class(Class::parse(ctx)?));
                }
                Token::Instance => {
                    items.push(Item::Instance(Instance::parse(ctx)?));
                }
                Token::Prototype => {
                    items.push(Item::Prototype(Prototype::parse(ctx)?));
                }
                Token::Const => {
                    items.push(Item::Const(Const::parse(ctx)?));
                    ctx.lexer.eat_token(Token::Semi)?;
                }
                Token::Var => {
                    items.push(Item::Var(Var::parse(ctx)?));
                    ctx.lexer.eat_token(Token::Semi)?;
                }
                Token::Func => {
                    items.push(Item::Func(FunctionDefinition::parse(ctx)?));
                }
                Token::Extern => {
                    items.push(Item::ExternFunc(ExternFunctionDefinition::parse(ctx)?));
                }
                Token::Eof => {
                    ctx.lexer.eat_token(Token::Eof)?;
                    break;
                }
                got => {
                    ctx.lexer.eat_any()?;
                    return Err(TokenError::unexpeced_token(got, ctx.lexer.span()).into());
                }
            }
        }

        Ok(Self { items })
    }
}
