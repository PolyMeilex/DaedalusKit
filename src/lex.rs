use logos::{Lexer, Logos};

use crate::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
pub enum Token {
    #[regex(r"([ \t])+")]
    Whitespace,
    #[regex(r"(\n|\r\n)+")]
    Newline,
    #[regex(r"//[^\n\r]*")]
    LineComment,

    #[token("const")]
    Const,
    #[token("var")]
    Var,
    #[token("instance")]
    Instance,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("func")]
    Func,
    #[token("prototype")]
    Prototype,
    #[token("null")]
    Null,
    #[token("class")]
    Class,
    #[token("return")]
    Return,

    #[regex("[-a-zA-Z_][a-zA-Z0-9_-]*", priority = 1)]
    Ident,
    #[regex("[+-]?[0-9_]+", priority = 2)]
    Integer,
    #[regex("\"", lex_string)]
    String,

    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token(";")]
    Semi,
    #[token("=")]
    Eq,
    #[token(",")]
    Comma,
    #[token("&")]
    And,
    #[token("|")]
    Or,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token(".")]
    Dot,
    #[token("!")]
    Bang,
    #[token("*")]
    Star,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Token::Whitespace => "whitespace",
            Token::Newline => "new line",
            Token::LineComment => "'//'",
            Token::Const => "'const'",
            Token::Var => "'var'",
            Token::Instance => "'instance'",
            Token::If => "'if'",
            Token::Else => "'else'",
            Token::Func => "'func'",
            Token::Prototype => "'prototype'",
            Token::Null => "'null'",
            Token::Class => "'class'",
            Token::Return => "'return'",
            Token::Ident => "identifier",
            Token::Integer => "intager",
            Token::String => "string",
            Token::OpenBrace => "'{'",
            Token::CloseBrace => "'}'",
            Token::OpenParen => "'('",
            Token::CloseParen => "')'",
            Token::OpenBracket => "'['",
            Token::CloseBracket => "']'",
            Token::Semi => "';'",
            Token::Eq => "'='",
            Token::Comma => "','",
            Token::And => "'&'",
            Token::Or => "'|'",
            Token::Lt => "'<'",
            Token::Gt => "'>'",
            Token::Plus => "'+'",
            Token::Dot => "'.'",
            Token::Bang => "'!'",
            Token::Star => "'*'",
        };
        write!(f, "{str}")
    }
}

fn lex_string(lex: &mut Lexer<Token>) -> bool {
    let remainder: &str = lex.remainder();
    let mut escaped = false;

    let mut total_len = 0;

    for c in remainder.chars() {
        total_len += c.len_utf8();

        if c == '\\' {
            escaped = !escaped;
            continue;
        }

        if c == '"' && !escaped {
            lex.bump(remainder[0..total_len].as_bytes().len());
            return true;
        }

        escaped = false;
    }
    false
}

#[derive(Clone)]
pub struct DaedalusLexer<'a> {
    lexer: Lexer<'a, Token>,
}

impl<'a> DaedalusLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Token::lexer(input),
        }
    }

    pub fn inner(&mut self) -> &mut Lexer<'a, Token> {
        &mut self.lexer
    }

    pub fn span(&self) -> logos::Span {
        self.lexer.span()
    }

    pub fn eat_whitespace(&mut self) {
        loop {
            let Some(Ok(token)) = self.lexer.clone().next() else {
                break;
            };

            if let Token::Whitespace | Token::Newline | Token::LineComment = token {
                self.lexer.next();
            } else {
                break;
            }
        }
    }

    pub fn peek(&mut self) -> Result<Token, ParseError> {
        self.eat_whitespace();

        let peek = self.lexer.clone().next();

        let Some(peek) = peek else {
            return Err(ParseError::eof(self.span()));
        };

        let Ok(peek) = peek else {
            return Err(ParseError::unkonown_token(self.span()));
        };

        Ok(peek)
    }

    pub fn eat_one(&mut self) -> Result<Token, ParseError> {
        self.eat_whitespace();

        let Some(token) = self.lexer.next() else {
            return Err(ParseError::eof(self.span()));
        };

        let Ok(token) = token else {
            return Err(ParseError::unkonown_token(self.span()));
        };

        Ok(token)
    }

    pub fn eat_ident(&mut self) -> Result<&'a str, ParseError> {
        self.eat_token(Token::Ident)?;
        Ok(self.lexer.slice())
    }

    pub fn eat_token(&mut self, expected: Token) -> Result<(), ParseError> {
        let got = self.eat_one()?;
        if got == expected {
            Ok(())
        } else {
            Err(ParseError::expected_token(got, expected, self.lexer.span()))
        }
    }
}
