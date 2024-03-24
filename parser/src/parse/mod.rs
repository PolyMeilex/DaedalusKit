use lexer::{DaedalusLexer, Token};

mod instance;
pub use instance::Instance;

mod var;
pub use var::Var;

mod func;
pub use func::FunctionDefinition;

mod func_call;
pub use func_call::FunctionCall;

mod if_statement;
pub use if_statement::IfStatement;

mod block;
pub use block::{Block, BlockItem};

mod return_statement;
pub use return_statement::ReturnStatement;

mod assign_statement;
pub use assign_statement::AssignStatement;

mod class;
pub use class::Class;

mod expr;
pub use expr::Expr;

mod ty;
pub use ty::Ty;

use crate::ParseError;

pub enum Item<'a> {
    Class(Class<'a>),
    Instance(Instance<'a>),
    Var(Var<'a>),
    Func(FunctionDefinition<'a>),
}

pub struct File<'a> {
    pub items: Vec<Item<'a>>,
}

impl<'a> File<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let mut items = Vec::new();

        while let Ok(token) = lexer.peek() {
            match token {
                Token::Class => {
                    items.push(Item::Class(Class::parse(lexer)?));
                }
                Token::Instance => {
                    items.push(Item::Instance(Instance::parse(lexer)?));
                }
                Token::Var => {
                    items.push(Item::Var(Var::parse(lexer)?));
                    lexer.eat_token(Token::Semi)?;
                }
                Token::Func => {
                    items.push(Item::Func(FunctionDefinition::parse(lexer)?));
                }
                _ => {
                    lexer.eat_any().unwrap();
                }
            }
        }

        Ok(Self { items })
    }
}
