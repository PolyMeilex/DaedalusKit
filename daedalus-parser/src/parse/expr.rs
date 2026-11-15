use daedalus_lexer::{DaedalusLexer, Token, TokenError};
use logos::Span;
use std::backtrace::Backtrace;

use crate::{DaedalusParser, ParseError};

use super::{FunctionCall, Ident};

#[derive(Debug, PartialEq, Eq)]
pub enum AssocOp {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `&&`
    And,
    /// `&`
    BitAnd,
    /// `||`
    Or,
    /// `|`
    BitOr,
    /// '*'
    Multiply,
    /// '/'
    Divide,
    /// `%`
    Modulo,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubtractAssign,
    /// `*=`
    MultiplyAssign,
    /// `/=`
    DivideAssign,
}

impl AssocOp {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::And => "&&",
            Self::BitAnd => "&",
            Self::Or => "||",
            Self::BitOr => "|",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::Assign => "=",
            Self::AddAssign => "+=",
            Self::SubtractAssign => "-=",
            Self::MultiplyAssign => "*=",
            Self::DivideAssign => "/=",
        }
    }

    fn parse_op(lexer: &mut DaedalusLexer) -> Result<Option<AssocOp>, ParseError> {
        let res = match lexer.peek()? {
            Token::Plus => {
                lexer.eat_token(Token::Plus)?;
                Self::Add
            }
            Token::Minus => {
                lexer.eat_token(Token::Minus)?;
                Self::Subtract
            }
            Token::EqEq => {
                lexer.eat_token(Token::EqEq)?;
                Self::Equal
            }
            Token::NotEq => {
                lexer.eat_token(Token::NotEq)?;
                Self::NotEqual
            }
            Token::Lt => {
                lexer.eat_token(Token::Lt)?;
                Self::Less
            }
            Token::Lte => {
                lexer.eat_token(Token::Lte)?;
                Self::LessEqual
            }
            Token::Gt => {
                lexer.eat_token(Token::Gt)?;
                Self::Greater
            }
            Token::Gte => {
                lexer.eat_token(Token::Gte)?;
                Self::GreaterEqual
            }
            Token::And => {
                lexer.eat_token(Token::And)?;
                Self::And
            }
            Token::BitAnd => {
                lexer.eat_token(Token::BitAnd)?;
                Self::BitAnd
            }
            Token::Or => {
                lexer.eat_token(Token::Or)?;
                Self::Or
            }
            Token::BitOr => {
                lexer.eat_token(Token::BitOr)?;
                Self::BitOr
            }
            Token::Star => {
                lexer.eat_token(Token::Star)?;
                Self::Multiply
            }
            Token::Slash => {
                lexer.eat_token(Token::Slash)?;
                Self::Divide
            }
            Token::Percent => {
                lexer.eat_token(Token::Percent)?;
                Self::Modulo
            }
            Token::ShiftLeft => {
                lexer.eat_token(Token::ShiftLeft)?;
                Self::ShiftLeft
            }
            Token::ShiftRight => {
                lexer.eat_token(Token::ShiftRight)?;
                Self::ShiftRight
            }
            Token::Eq => {
                lexer.eat_token(Token::Eq)?;
                Self::Assign
            }
            Token::PlusEq => {
                lexer.eat_token(Token::PlusEq)?;
                Self::AddAssign
            }
            Token::MinusEq => {
                lexer.eat_token(Token::MinusEq)?;
                Self::SubtractAssign
            }
            Token::StarEq => {
                lexer.eat_token(Token::StarEq)?;
                Self::MultiplyAssign
            }
            Token::SlashEq => {
                lexer.eat_token(Token::SlashEq)?;
                Self::DivideAssign
            }
            _ => return Ok(None),
        };

        Ok(Some(res))
    }

    fn priority(&self) -> u32 {
        match self {
            AssocOp::Multiply | AssocOp::Divide | AssocOp::Modulo => 10,
            AssocOp::Add | AssocOp::Subtract => 9,
            AssocOp::ShiftLeft | AssocOp::ShiftRight => 8,

            AssocOp::Less | AssocOp::LessEqual | AssocOp::Greater | AssocOp::GreaterEqual => 7,
            AssocOp::Equal | AssocOp::NotEqual => 6,

            AssocOp::BitAnd => 5,
            AssocOp::BitOr => 4,

            AssocOp::And => 3,
            AssocOp::Or => 2,

            AssocOp::Assign
            | AssocOp::AddAssign
            | AssocOp::SubtractAssign
            | AssocOp::MultiplyAssign
            | AssocOp::DivideAssign => 1,
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    /// '!'
    Not,
    /// '-'
    Negative,
    /// '~', bitwise complement
    Complement,
}

#[derive(Debug)]
pub enum LitKind {
    Intager(i32),
    Float(f32),
    String(String),
}

#[derive(Debug)]
pub struct Lit {
    pub kind: LitKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Binary(AssocOp, Box<Expr>, Box<Expr>),
    /// For not only `!`/`-` unary op
    Unary(UnaryOp, Box<Expr>),

    Lit(Lit),
    Call(FunctionCall),
    Ident(Ident),
    /// (a)
    Paren(Box<Expr>),
    /// a.b
    Field(Box<Expr>, Ident),
    /// a[b]
    Index(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn parse(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        ctx.lexer.eat_whitespace();

        let left = Self::parse_without_op(ctx)?;
        let expr = Self::parse_with_op(ctx, left, 0)?;

        Ok(expr)
    }

    fn parse_with_op(
        ctx: &mut DaedalusParser,
        mut left: Self,
        left_priority: u32,
    ) -> Result<Self, ParseError> {
        while let Some(op) = AssocOp::parse_op(&mut ctx.lexer.clone())? {
            let priority = op.priority();

            if priority < left_priority {
                break;
            };

            AssocOp::parse_op(ctx.lexer)?;

            let right = Self::parse_without_op(ctx)?;
            let res = Self::parse_with_op(ctx, right, priority + 1)?;

            left = Self {
                span: left.span.start..res.span.end,
                kind: ExprKind::Binary(op, Box::new(left), Box::new(res)),
            };
        }

        Ok(left)
    }

    fn parse_without_op(ctx: &mut DaedalusParser) -> Result<Self, ParseError> {
        let mut peek_lexer = ctx.lexer.clone();
        let token = peek_lexer.peek()?;
        let start = {
            let mut lexer = peek_lexer.clone();
            lexer.eat_any()?;
            lexer.span().start
        };

        let kind = match token {
            Token::Bang => {
                ctx.lexer.eat_token(Token::Bang)?;
                let expr = Self::parse_without_op(ctx)?;
                ExprKind::Unary(UnaryOp::Not, Box::new(expr))
            }
            Token::Tilde => {
                ctx.lexer.eat_token(Token::Tilde)?;
                let expr = Self::parse_without_op(ctx)?;
                ExprKind::Unary(UnaryOp::Complement, Box::new(expr))
            }
            Token::Minus => {
                ctx.lexer.eat_token(Token::Minus)?;
                let expr = Self::parse_without_op(ctx)?;
                ExprKind::Unary(UnaryOp::Negative, Box::new(expr))
            }
            Token::String => {
                let raw = ctx.lexer.eat_token(Token::String)?;
                ExprKind::Lit(Lit {
                    kind: LitKind::String(raw.to_string()),
                })
            }
            Token::Integer => {
                let raw = ctx.lexer.eat_token(Token::Integer)?;
                let value = raw.parse::<i32>().map_err(|err| ParseError::IntLitError {
                    err,
                    span: ctx.lexer.span(),
                    backtrace: Backtrace::capture(),
                })?;
                ExprKind::Lit(Lit {
                    kind: LitKind::Intager(value),
                })
            }
            Token::Float => {
                let raw = ctx.lexer.eat_token(Token::Float)?;
                let value = raw
                    .parse::<f32>()
                    .map_err(|err| ParseError::FloatLitError {
                        err,
                        span: ctx.lexer.span(),
                        backtrace: Backtrace::capture(),
                    })?;
                ExprKind::Lit(Lit {
                    kind: LitKind::Float(value),
                })
            }
            Token::Ident => {
                peek_lexer.eat_token(Token::Ident)?;

                let kind = match peek_lexer.peek()? {
                    Token::OpenParen => ExprKind::Call(FunctionCall::parse(ctx)?),
                    _ => ExprKind::Ident(Ident::parse(ctx)?),
                };

                let expr = Expr {
                    kind,
                    span: start..ctx.lexer.span().end,
                };
                return Self::parse_reference(ctx, expr);
            }
            Token::OpenParen => {
                ctx.lexer.eat_token(Token::OpenParen)?;
                let expr = Expr::parse(ctx)?;
                ctx.lexer.eat_token(Token::CloseParen)?;
                ExprKind::Paren(Box::new(expr))
            }
            got => {
                peek_lexer.eat_any()?;
                return Err(TokenError::unexpeced_token(got, peek_lexer.span()).into());
            }
        };

        Ok(Expr {
            kind,
            span: start..ctx.lexer.span().end,
        })
    }

    pub fn parse_reference(
        ctx: &mut DaedalusParser,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        let expr = if ctx.lexer.peek()? == Token::OpenBracket {
            Self::parse_array_index(ctx, parent_expr)?
        } else {
            parent_expr
        };

        if ctx.lexer.peek()? == Token::Dot {
            let expr = Self::parse_field_access(ctx, expr)?;
            Self::parse_reference(ctx, expr)
        } else {
            Ok(expr)
        }
    }

    pub fn parse_array_index(
        ctx: &mut DaedalusParser,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::OpenBracket)?;
        let index = Expr::parse(ctx)?;
        ctx.lexer.eat_token(Token::CloseBracket)?;

        Ok(Expr {
            span: parent_expr.span.start..ctx.lexer.span().end,
            kind: ExprKind::Index(Box::new(parent_expr), Box::new(index)),
        })
    }

    pub fn parse_field_access(
        ctx: &mut DaedalusParser,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        ctx.lexer.eat_token(Token::Dot)?;
        let ident = Ident::parse(ctx)?;
        Ok(Expr {
            span: parent_expr.span.start..ctx.lexer.span().end,
            kind: ExprKind::Field(Box::new(parent_expr), ident),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daedalus_lexer::DaedalusLexer;
    use indoc::indoc;

    fn parse(src: &str) -> Expr {
        Expr::parse(&mut DaedalusParser {
            lexer: &mut DaedalusLexer::new(src),
        })
        .unwrap()
    }

    #[test]
    fn spans() {
        let src = "  a + foo(1)[2].b";
        let expr = parse(src);
        assert_eq!(&src[expr.span.clone()], "a + foo(1)[2].b");

        let ExprKind::Binary(_, left, right) = &expr.kind else {
            panic!("{expr:?}");
        };
        assert_eq!(&src[left.span.clone()], "a");
        assert_eq!(&src[right.span.clone()], "foo(1)[2].b");

        let ExprKind::Field(indexed, field) = &right.kind else {
            panic!("{right:?}");
        };
        assert_eq!(&src[indexed.span.clone()], "foo(1)[2]");
        assert_eq!(&src[field.span.clone()], "b");

        let src = "-(1 + 2)";
        let expr = parse(src);
        assert_eq!(&src[expr.span.clone()], src);
        let ExprKind::Unary(_, paren) = &expr.kind else {
            panic!("{expr:?}");
        };
        assert_eq!(&src[paren.span.clone()], "(1 + 2)");
    }

    #[test]
    #[ignore]
    fn assoc_op() {
        let src = indoc! {"
        1 + 2 * 3 + 4
        "};

        let expr = Expr::parse(&mut DaedalusParser {
            lexer: &mut DaedalusLexer::new(src),
        })
        .unwrap();
        dbg!(expr);
    }
}
