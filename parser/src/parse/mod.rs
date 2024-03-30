use lexer::{DaedalusLexer, Token};

mod instance;
pub use instance::Instance;

mod var;
pub use var::Var;

mod const_def;
pub use const_def::Const;

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

mod class;
pub use class::Class;

mod expr;
pub use expr::Expr;

mod ty;
pub use ty::Ty;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

#[derive(Debug)]
pub enum Item<'a> {
    Class(Class<'a>),
    Instance(Instance<'a>),
    Var(Var<'a>),
    Const(Const<'a>),
    Func(FunctionDefinition<'a>),
}

pub struct File<'a> {
    pub items: Vec<Item<'a>>,
}

impl<'a> DaedalusDisplay for File<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        for item in &self.items {
            match item {
                Item::Class(v) => {
                    v.fmt(f)?;
                }
                Item::Instance(v) => {
                    v.fmt(f)?;
                }
                Item::Var(v) => {
                    v.fmt(f)?;
                }
                Item::Func(v) => {
                    v.fmt(f)?;
                }
                Item::Const(v) => {
                    v.fmt(f)?;
                }
            }
        }

        Ok(())
    }
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
                Token::Const => {
                    items.push(Item::Const(Const::parse(lexer)?));
                    lexer.eat_token(Token::Semi)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn diff(src: &str) {
        let ast = File::parse(&mut DaedalusLexer::new(src)).unwrap();
        let mut out = String::new();
        DaedalusFormatter::new(&mut out).format(ast).unwrap();
        assert_eq!(src.trim_end(), out.trim_end());
    }

    macro_rules! diff {
        ($($arg:tt)*) => {
            diff(indoc::indoc!($($arg)*))
        };
    }

    #[test]
    fn func() {
        diff!("func void a() {};");
        diff!("func int b() {};");
        diff!("func string c(var int a) {};");
        diff!("func float d(var int a, var int b) {};");
        diff!("func func e(var func a, var int b) {};");
    }

    #[test]
    fn expr() {
        diff! {"
            func int a() {
                a = abc[1].cba[2].xyz.abc;
            };
        "};
        diff! {"
            func int a() {
                abc[1].cba[2].xyz.abc[2] = 5;
            };
        "};
        diff! {"
            func int a() {
                abc[A].cba[B].xyz.abc[C] = \"test\";
            };
        "};
        diff! {"
            func int a() {
                b = \"test\" + 1.5;
            };
        "};
        diff! {"
            func int a() {
                c = \"test\" + 1;
            };
        "};
    }
}
