use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

use super::FunctionCall;

#[derive(Debug)]
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
    /// `!`
    Not,
    /// '*'
    Multiply,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
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
            Self::Not => "!",
            Self::Multiply => "*",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
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
            Token::ShiftLeft => {
                lexer.eat_token(Token::ShiftLeft)?;
                Self::ShiftLeft
            }
            Token::ShiftRight => {
                lexer.eat_token(Token::ShiftRight)?;
                Self::ShiftRight
            }
            _ => return Ok(None),
        };

        Ok(Some(res))
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    /// '!'
    Not,
    /// '-'
    Negative,
}

#[derive(Debug)]
pub struct Lit<'a> {
    raw: &'a str,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    Binary(AssocOp, Box<Expr<'a>>, Box<Expr<'a>>),
    // For not only `!`/`-` unary op
    Unary(UnaryOp, Box<Expr<'a>>),

    Lit(Lit<'a>),
    Call(FunctionCall<'a>),
    Ident(&'a str),
    /// (a)
    Paren(Box<Expr<'a>>),
    /// a.b
    Field(Box<Expr<'a>>, &'a str),
    /// a[b]
    Index(Box<Expr<'a>>, Box<Expr<'a>>),
}

#[derive(Debug)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
}

impl<'a> DaedalusDisplay for Expr<'a> {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        match &self.kind {
            ExprKind::Binary(op, left, right) => {
                left.fmt(f)?;
                write!(f, " {} ", op.as_str())?;
                right.fmt(f)?;
            }
            ExprKind::Unary(op, v) => {
                match op {
                    UnaryOp::Not => write!(f, "!")?,
                    UnaryOp::Negative => write!(f, "-")?,
                }
                v.fmt(f)?;
            }
            ExprKind::Lit(lit) => {
                write!(f, "{}", lit.raw)?;
            }
            ExprKind::Call(call) => {
                call.fmt(f)?;
            }
            ExprKind::Ident(i) => {
                write!(f, "{i}")?;
            }
            ExprKind::Paren(p) => {
                write!(f, "(")?;
                p.fmt(f)?;
                write!(f, ")")?;
            }
            ExprKind::Field(obj, field) => {
                obj.fmt(f)?;
                write!(f, ".{field}")?;
            }
            ExprKind::Index(a, b) => {
                a.fmt(f)?;
                write!(f, "[")?;
                b.fmt(f)?;
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

impl<'a> Expr<'a> {
    pub fn parse(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        lexer.eat_whitespace();

        let left = Self::parse_without_op(lexer)?;
        let expr = Self::parse_with_op(lexer, left)?;

        Ok(expr)
    }

    fn parse_with_op(lexer: &mut DaedalusLexer<'a>, left: Self) -> Result<Self, ParseError> {
        if let Some(op) = AssocOp::parse_op(lexer)? {
            let right = Self::parse_without_op(lexer)?;

            let next_left = Self {
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            };

            Self::parse_with_op(lexer, next_left)
        } else {
            Ok(left)
        }
    }

    fn parse_without_op(lexer: &mut DaedalusLexer<'a>) -> Result<Self, ParseError> {
        let mut peek_lexer = lexer.clone();
        let kind = match peek_lexer.peek()? {
            Token::Bang => {
                lexer.eat_token(Token::Bang)?;
                let expr = Self::parse_without_op(lexer)?;
                ExprKind::Unary(UnaryOp::Not, Box::new(expr))
            }
            Token::Minus => {
                lexer.eat_token(Token::Minus)?;
                let expr = Self::parse_without_op(lexer)?;
                ExprKind::Unary(UnaryOp::Negative, Box::new(expr))
            }
            Token::String => {
                let raw = lexer.eat_token(Token::String)?;
                ExprKind::Lit(Lit { raw })
            }
            Token::Integer => {
                let raw = lexer.eat_token(Token::Integer)?;
                ExprKind::Lit(Lit { raw })
            }
            Token::Float => {
                let raw = lexer.eat_token(Token::Float)?;
                ExprKind::Lit(Lit { raw })
            }
            Token::Ident => {
                peek_lexer.eat_token(Token::Ident)?;

                let expr = match peek_lexer.peek()? {
                    Token::OpenParen => {
                        let call = FunctionCall::parse(lexer)?;
                        ExprKind::Call(call)
                    }
                    _ => {
                        let ident = lexer.eat_token(Token::Ident)?;
                        ExprKind::Ident(ident)
                    }
                };

                let expr = if lexer.peek()? == Token::OpenBracket {
                    lexer.eat_token(Token::OpenBracket)?;
                    let index = Expr::parse(lexer)?;
                    lexer.eat_token(Token::CloseBracket)?;
                    ExprKind::Index(Box::new(Expr { kind: expr }), Box::new(index))
                } else {
                    expr
                };

                match peek_lexer.peek()? {
                    Token::Dot => {
                        lexer.eat_token(Token::Dot)?;
                        let ident = lexer.eat_token(Token::Ident)?;
                        let expr = ExprKind::Field(Box::new(Expr { kind: expr }), ident);

                        if lexer.peek()? == Token::OpenBracket {
                            lexer.eat_token(Token::OpenBracket)?;
                            let index = Expr::parse(lexer)?;
                            lexer.eat_token(Token::CloseBracket)?;
                            ExprKind::Index(Box::new(Expr { kind: expr }), Box::new(index))
                        } else {
                            expr
                        }
                    }
                    _ => expr,
                }
            }
            Token::OpenParen => {
                lexer.eat_token(Token::OpenParen)?;
                let expr = Expr::parse(lexer)?;
                lexer.eat_token(Token::CloseParen)?;
                ExprKind::Paren(Box::new(expr))
            }
            got => {
                peek_lexer.eat_any()?;
                return Err(ParseError::unexpeced_token(got, peek_lexer.span()));
            }
        };

        Ok(Expr { kind })
    }
}
