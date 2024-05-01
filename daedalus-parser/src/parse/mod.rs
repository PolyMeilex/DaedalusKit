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

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    DaedalusParser, ParseError,
};

use std::fmt::Write;

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

impl DaedalusDisplay for File {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        for item in &self.items {
            match item {
                Item::Class(v) => {
                    v.fmt(f)?;
                }
                Item::Instance(v) => {
                    v.fmt(f)?;
                }
                Item::Prototype(v) => {
                    v.fmt(f)?;
                }
                Item::Var(v) => {
                    v.fmt(f)?;
                    writeln!(f, ";")?;
                }
                Item::Const(v) => {
                    v.fmt(f)?;
                    writeln!(f, ";")?;
                }
                Item::Func(v) => {
                    v.fmt(f)?;
                }
                Item::ExternFunc(v) => {
                    v.fmt(f)?;
                }
            }
        }

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use daedalus_lexer::DaedalusLexer;
    use pretty_assertions::assert_eq;

    fn diff(src: &str) {
        let ast = File::parse(&mut DaedalusParser {
            lexer: &mut DaedalusLexer::new(src),
        })
        .unwrap();
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
    fn extern_func() {
        diff!("extern func void a();");
        diff!("extern func int b();");
        diff!("extern func string c(var int a);");
        diff!("extern func float d(var int a, var int b);");
        diff!("extern func func e(var func a, var int b);");
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
