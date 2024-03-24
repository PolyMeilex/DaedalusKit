use logos::{Lexer, Logos};

#[derive(Debug, thiserror::Error)]
pub enum TokenErrorKind {
    #[error("Unkonown token")]
    UnkonownToken,
    #[error("Unexpected {got}")]
    UnexpecedToken { got: Token },
    #[error("Expected {expected} got {got}")]
    ExpectedToken { expected: Token, got: Token },
    #[error("Unexpected end of file")]
    EOF,
}

#[derive(Debug, thiserror::Error)]
#[error("{kind} {span:?}")]
pub struct TokenError {
    pub kind: TokenErrorKind,
    pub span: logos::Span,
}

impl TokenError {
    pub fn eof(span: logos::Span) -> Self {
        Self {
            kind: TokenErrorKind::EOF,
            span,
        }
    }

    pub fn unkonown_token(span: logos::Span) -> Self {
        Self {
            kind: TokenErrorKind::UnkonownToken,
            span,
        }
    }

    pub fn unexpeced_token(got: Token, span: logos::Span) -> Self {
        Self {
            kind: TokenErrorKind::UnexpecedToken { got },
            span,
        }
    }

    pub fn expected_token(got: Token, expected: Token, span: logos::Span) -> Self {
        Self {
            kind: TokenErrorKind::ExpectedToken { got, expected },
            span,
        }
    }

    pub fn span(&self) -> &logos::Span {
        &self.span
    }
}

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

    #[regex(r"(\p{XID_Start}|_)\p{XID_Continue}*", priority = 1)]
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
    #[token("-")]
    Minus,
    #[token(".")]
    Dot,
    #[token("!")]
    Bang,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
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
            Token::Minus => "'-'",
            Token::Dot => "'.'",
            Token::Bang => "'!'",
            Token::Star => "'*'",
            Token::Slash => "/",
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

    pub fn eat_while(&mut self, f: impl Fn(&Token) -> bool) {
        loop {
            let Some(Ok(token)) = self.lexer.clone().next() else {
                break;
            };

            if f(&token) {
                self.lexer.next();
            } else {
                break;
            }
        }
    }

    pub fn eat_whitespace(&mut self) {
        self.eat_while(|token| {
            matches!(
                token,
                Token::Whitespace | Token::Newline | Token::LineComment
            )
        });
    }

    pub fn peek_raw(&mut self) -> Result<Token, TokenError> {
        self.clone().eat_one_raw()
    }

    pub fn eat_one_raw(&mut self) -> Result<Token, TokenError> {
        let Some(token) = self.lexer.next() else {
            return Err(TokenError::eof(self.span()));
        };

        let Ok(token) = token else {
            return Err(TokenError::unkonown_token(self.span()));
        };

        Ok(token)
    }

    pub fn eat_token_raw(&mut self, expected: Token) -> Result<(), TokenError> {
        let got = self.eat_one_raw()?;
        if got == expected {
            Ok(())
        } else {
            Err(TokenError::expected_token(got, expected, self.lexer.span()))
        }
    }

    pub fn peek_with_comments(&mut self) -> Result<Token, TokenError> {
        self.eat_while(|token| matches!(token, Token::Whitespace | Token::Newline));
        self.peek_raw()
    }

    pub fn peek(&mut self) -> Result<Token, TokenError> {
        self.eat_whitespace();
        self.peek_raw()
    }

    pub fn eat_one(&mut self) -> Result<Token, TokenError> {
        self.eat_whitespace();
        self.eat_one_raw()
    }

    pub fn eat_line_comment(&mut self) -> Result<&'a str, TokenError> {
        self.eat_token_raw(Token::LineComment)?;
        let str = &self.lexer.slice()[2..];
        Ok(str)
    }

    pub fn eat_token(&mut self, expected: Token) -> Result<&'a str, TokenError> {
        let got = self.eat_one()?;
        if got == expected {
            if expected == Token::String {
                let str = &self.lexer.slice()[1..];
                let str = &str[..str.len() - 1];
                Ok(str)
            } else {
                Ok(self.lexer.slice())
            }
        } else {
            Err(TokenError::expected_token(got, expected, self.lexer.span()))
        }
    }
}
