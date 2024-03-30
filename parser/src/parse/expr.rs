use lexer::{DaedalusLexer, Token};
use std::fmt::Write;

use crate::{
    fmt::{DaedalusDisplay, DaedalusFormatter},
    ParseError,
};

use super::{FunctionCall, Ident};

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
    /// '/'
    Divide,
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
            Self::Not => "!",
            Self::Multiply => "*",
            Self::Divide => "/",
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
}

#[derive(Debug)]
pub enum UnaryOp {
    /// '!'
    Not,
    /// '-'
    Negative,
}

#[derive(Debug)]
pub enum LitKind<'a> {
    Intager(&'a str),
    Float(&'a str),
    String(&'a str),
}

#[derive(Debug)]
pub struct Lit<'a> {
    kind: LitKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    Binary(AssocOp, Box<Expr<'a>>, Box<Expr<'a>>),
    /// For not only `!`/`-` unary op
    Unary(UnaryOp, Box<Expr<'a>>),

    Lit(Lit<'a>),
    Call(FunctionCall<'a>),
    Ident(Ident<'a>),
    /// (a)
    Paren(Box<Expr<'a>>),
    /// a.b
    Field(Box<Expr<'a>>, Ident<'a>),
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
            ExprKind::Lit(Lit {
                kind: LitKind::String(lit),
            }) => {
                write!(f, "\"{}\"", lit)?;
            }
            ExprKind::Lit(Lit {
                kind: LitKind::Intager(lit),
            }) => {
                write!(f, "{}", lit)?;
            }
            ExprKind::Lit(Lit {
                kind: LitKind::Float(lit),
            }) => {
                write!(f, "{}", lit)?;
            }
            ExprKind::Call(call) => {
                call.fmt(f)?;
            }
            ExprKind::Ident(i) => {
                i.fmt(f)?;
            }
            ExprKind::Paren(p) => {
                write!(f, "(")?;
                p.fmt(f)?;
                write!(f, ")")?;
            }
            ExprKind::Field(obj, field) => {
                obj.fmt(f)?;
                write!(f, ".")?;
                field.fmt(f)?;
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
        let expr = match peek_lexer.peek()? {
            Token::Bang => {
                lexer.eat_token(Token::Bang)?;
                let expr = Self::parse_without_op(lexer)?;
                Expr {
                    kind: ExprKind::Unary(UnaryOp::Not, Box::new(expr)),
                }
            }
            Token::Minus => {
                lexer.eat_token(Token::Minus)?;
                let expr = Self::parse_without_op(lexer)?;
                Expr {
                    kind: ExprKind::Unary(UnaryOp::Negative, Box::new(expr)),
                }
            }
            Token::String => {
                let raw = lexer.eat_token(Token::String)?;
                Expr {
                    kind: ExprKind::Lit(Lit {
                        kind: LitKind::String(raw),
                    }),
                }
            }
            Token::Integer => {
                let raw = lexer.eat_token(Token::Integer)?;
                Expr {
                    kind: ExprKind::Lit(Lit {
                        kind: LitKind::Intager(raw),
                    }),
                }
            }
            Token::Float => {
                let raw = lexer.eat_token(Token::Float)?;
                Expr {
                    kind: ExprKind::Lit(Lit {
                        kind: LitKind::Float(raw),
                    }),
                }
            }
            Token::Ident => {
                peek_lexer.eat_token(Token::Ident)?;

                let expr = match peek_lexer.peek()? {
                    Token::OpenParen => {
                        let call = FunctionCall::parse(lexer)?;
                        Expr {
                            kind: ExprKind::Call(call),
                        }
                    }
                    _ => {
                        let ident = Ident::parse(lexer)?;
                        Expr {
                            kind: ExprKind::Ident(ident),
                        }
                    }
                };

                Self::parse_reference(lexer, expr)?
            }
            Token::OpenParen => {
                lexer.eat_token(Token::OpenParen)?;
                let expr = Expr::parse(lexer)?;
                lexer.eat_token(Token::CloseParen)?;
                Expr {
                    kind: ExprKind::Paren(Box::new(expr)),
                }
            }
            got => {
                peek_lexer.eat_any()?;
                return Err(ParseError::unexpeced_token(got, peek_lexer.span()));
            }
        };

        Ok(expr)
    }

    pub fn parse_reference(
        lexer: &mut DaedalusLexer<'a>,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        let expr = if lexer.peek()? == Token::OpenBracket {
            Self::parse_array_index(lexer, parent_expr)?
        } else {
            parent_expr
        };

        if lexer.peek()? == Token::Dot {
            let expr = Self::parse_field_access(lexer, expr)?;
            Self::parse_reference(lexer, expr)
        } else {
            Ok(expr)
        }
    }

    pub fn parse_array_index(
        lexer: &mut DaedalusLexer<'a>,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        lexer.eat_token(Token::OpenBracket)?;
        let index = Expr::parse(lexer)?;
        lexer.eat_token(Token::CloseBracket)?;

        Ok(Expr {
            kind: ExprKind::Index(Box::new(parent_expr), Box::new(index)),
        })
    }

    pub fn parse_field_access(
        lexer: &mut DaedalusLexer<'a>,
        parent_expr: Self,
    ) -> Result<Self, ParseError> {
        lexer.eat_token(Token::Dot)?;
        let ident = Ident::parse(lexer)?;
        Ok(Expr {
            kind: ExprKind::Field(Box::new(parent_expr), ident),
        })
    }
}
