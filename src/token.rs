use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    Comma,
    Colon,
    Hash,

    Comment,
    Whitespace,
    Newline,

    False,
    True,
    Ident,
    Number,
    String,
    None,

    Eof,
}

impl TokenKind {
    pub fn str(&self) -> &'static str {
        match self {
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::Hash => "#",
            TokenKind::Comment => "<COMMENT>",
            TokenKind::Whitespace => "\\s",
            TokenKind::Newline => "\\n",
            TokenKind::False => "false",
            TokenKind::True => "true",
            TokenKind::None => "None",
            TokenKind::Number => "<NUMBER>",
            TokenKind::Eof => "<EOF>",
            TokenKind::Ident => "<IDENT>",
            TokenKind::String => "<STRING>",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        match self.kind {
            False | True => {
                write!(f, "keyword {}", self.kind.str())
            }
            Ident => write!(f, "identifier {}", self.text),
            _ => write!(f, "{}", self.kind.str()),
        }
    }
}
