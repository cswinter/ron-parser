use std::fmt;
use std::iter::Peekable;
use std::ops::Range;
use std::str::Chars;

use ariadne::{Label, Report, ReportBuilder, ReportKind};

use crate::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    errors: Vec<ReportBuilder<Range<usize>>>,
    chars: Vec<char>,

    start: usize,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            source: source.chars().peekable(),

            tokens: vec![],
            errors: vec![],
            chars: vec![],

            start: 0,
            current: 0,
        }
    }

    pub fn scan(mut self) -> (Vec<Token>, Vec<ReportBuilder<Range<usize>>>) {
        loop {
            match self.scan_token() {
                Ok(TokenKind::Whitespace) | Ok(TokenKind::Comment) | Ok(TokenKind::Newline) => {}
                Ok(token) => {
                    let mut span = Token {
                        kind: token,
                        span: Span {
                            start: self.start,
                            end: self.current,
                        },
                        text: self.chars.iter().collect(),
                    };

                    if token == TokenKind::Ident {
                        match span.text.as_ref() {
                            "true" => span.kind = TokenKind::True,
                            "false" => span.kind = TokenKind::False,
                            "None" => span.kind = TokenKind::None,
                            _ => {}
                        }
                    }

                    self.tokens.push(span);

                    if let TokenKind::Eof = token {
                        break;
                    }
                }
                Err(err) => {
                    self.errors.push(err);
                }
            }
            self.chars.clear();
            self.start = self.current;
        }
        (self.tokens, self.errors)
    }

    fn scan_token(&mut self) -> Result<TokenKind, ReportBuilder<Range<usize>>> {
        let token = match self.advance() {
            None => TokenKind::Eof,
            Some(c) => match c {
                '(' => TokenKind::LeftParen,
                ')' => TokenKind::RightParen,
                '{' => TokenKind::LeftBrace,
                '}' => TokenKind::RightBrace,
                '[' => TokenKind::LeftBracket,
                ']' => TokenKind::RightBracket,
                ':' => TokenKind::Colon,
                ',' => TokenKind::Comma,
                '/' => {
                    if self.consume('/') {
                        while self.peek().is_some() && self.peek() != Some('\n') {
                            self.advance();
                        }
                        TokenKind::Comment
                    } else if self.consume('*') {
                        let previous = ' ';
                        while self.peek().is_some() && (self.peek() != Some('/') || previous != '*')
                        {
                            self.advance();
                        }
                        TokenKind::Comment
                    } else {
                        todo!()
                    }
                }
                ' ' | '\r' | '\t' => TokenKind::Whitespace,
                '\n' => TokenKind::Newline,
                '0'..='9' | '-' => {
                    self.number();
                    TokenKind::Number
                }
                '"' => {
                    self.string();
                    TokenKind::String
                }
                '_' | 'a'..='z' | 'A'..='Z' => {
                    self.ident();
                    TokenKind::Ident
                }
                t => {
                    return Err(Report::build(ReportKind::Error, (), self.current)
                        .with_message(format!("Unexpected character `{}`", t))
                        .with_label(Label::new(self.current - 1..self.current)));
                }
            },
        };
        Ok(token)
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.next();
        self.current += 1;
        if let Some(c) = c {
            self.chars.push(c);
        }
        c
    }

    fn number(&mut self) {
        while let Some('0'..='9' | '-' | '+' | '.' | 'e') = self.peek() {
            self.advance();
        }
    }

    fn string(&mut self) {
        let mut escaped = false;
        while let Some(c) = self.advance() {
            if c == '"' && !escaped {
                break;
            }
            escaped = c == '\\';
        }
    }

    fn ident(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.source.peek().cloned()
    }
}

pub struct TokenStream<'a>(pub &'a [Token]);

impl<'a> fmt::Display for TokenStream<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for token in self.0 {
            write!(f, "{} ", token.text)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer() {
        use TokenKind::*;
        let lexer = Lexer::new(
            r#"
Config(
    version: 1,
)"#,
        );
        let (tokens, errs) = lexer.scan();
        assert_eq!(errs.len(), 0);
        assert_eq!(
            &tokens,
            &[
                Token {
                    kind: Ident,
                    span: Span { start: 1, end: 7 },
                    text: "Config".to_string(),
                },
                Token {
                    kind: LeftParen,
                    span: Span { start: 7, end: 8 },
                    text: "(".to_string(),
                },
                Token {
                    kind: Ident,
                    span: Span { start: 13, end: 20 },
                    text: "version".to_string(),
                },
                Token {
                    kind: Colon,
                    span: Span { start: 20, end: 21 },
                    text: ":".to_string(),
                },
                Token {
                    kind: Number,
                    span: Span { start: 22, end: 23 },
                    text: "1".to_string(),
                },
                Token {
                    kind: Comma,
                    span: Span { start: 23, end: 24 },
                    text: ",".to_string(),
                },
                Token {
                    kind: RightParen,
                    span: Span { start: 25, end: 26 },
                    text: ")".to_string(),
                },
                Token {
                    kind: Eof,
                    span: Span { start: 26, end: 27 },
                    text: "".to_string(),
                }
            ]
        );
    }
}
