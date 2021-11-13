use std::ops::Range;

use ariadne::{Label, Report, ReportBuilder, ReportKind};
use indexmap::IndexMap;

use crate::token::{Token, TokenKind};
use crate::value::{Map, Number, Struct, Value};
use crate::Lexer;

type Result<T> = std::result::Result<T, ReportBuilder<Range<usize>>>;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    current: usize,
    errors: Vec<ReportBuilder<Range<usize>>>,
}

impl Parser {
    pub fn new(source: &str) -> Parser {
        let (tokens, errors) = Lexer::new(source).scan();
        Parser {
            tokens,
            current: 0,
            errors,
        }
    }

    pub fn parse(mut self) -> (Value, Vec<ReportBuilder<Range<usize>>>) {
        let value = self.value();
        if !self.is_at_end() {
            self.errors.push(
                Report::build(ReportKind::Error, (), self.pos())
                    .with_message("Expected end of input.")
                    .with_label(Label::new(self.pos()..self.pos()).with_message("")),
            );
        }
        (value, self.errors)
    }

    fn value(&mut self) -> Value {
        let val = match self.peek().kind {
            TokenKind::Ident => {
                let start = self.peek().span.start;
                let name = self.advance().text.clone();
                self.struct_or_tuple(start, Some(name))
            }
            TokenKind::LeftParen => self.struct_or_tuple(self.peek().span.start, None),
            TokenKind::LeftBrace => self.map(),
            TokenKind::LeftBracket => self.seq(),
            TokenKind::False => {
                self.advance();
                Ok(Value::Bool(false))
            }
            TokenKind::True => {
                self.advance();
                Ok(Value::Bool(true))
            }
            TokenKind::None => {
                self.advance();
                Ok(Value::Option(None))
            }
            TokenKind::Number => {
                let text = self.advance().text.clone();
                match text.parse::<i64>() {
                    Ok(int) => Ok(Value::Number(Number::Integer(int))),
                    Err(_) => match text.parse::<f64>() {
                        Ok(float) => Ok(Value::Number(Number::from(float))),
                        Err(err) => {
                            Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                                .with_message(format!("Malformed number `{}`", text))
                                .with_label(
                                    Label::new(self.peek().span.start..self.peek().span.end)
                                        .with_message(""),
                                )
                                .with_note(format!("Failed to parse number: {}", err)))
                        }
                    },
                }
            }
            TokenKind::String => {
                // TODO: unicode escapes, 7bit character codes
                let mut string = String::new();
                let mut escaped = false;
                let text = self.advance().text.clone();
                for (i, char) in text[1..text.len() - 1].chars().enumerate() {
                    if escaped {
                        match char {
                            'n' => string.push('\n'),
                            'r' => string.push('\r'),
                            't' => string.push('\t'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            '0' => string.push('\0'),
                            _ => {
                                self.errors.push(
                                    Report::build(
                                        ReportKind::Error,
                                        (),
                                        self.peek().span.start + i - 1,
                                    )
                                    .with_message(format!("unknown character escape: `\\{}`", char))
                                    .with_label(
                                        Label::new(
                                            self.peek().span.start + i - 1
                                                ..self.peek().span.start + i,
                                        )
                                        .with_message(""),
                                    )
                                    .with_note("Valid escape sequences are: `\\n`, `\\r`, `\\t`, `\\\"`, `\\0`"),
                                );
                            }
                        }
                        escaped = false;
                    } else if char == '\\' {
                        escaped = true;
                    } else {
                        string.push(char);
                    }
                }
                Ok(Value::String(string))
            }
            TokenKind::Eof => todo!(),
            _ => todo!(),
        };
        match val {
            Ok(val) => val,
            Err(err) => {
                self.errors.push(err);
                Value::Unit
            }
        }
    }

    fn struct_or_tuple(&mut self, start: usize, name: Option<String>) -> Result<Value> {
        if self.check2(TokenKind::Ident) && self.check3(TokenKind::Colon) {
            self.structure(start, name)
        } else {
            self.tuple(start)
        }
    }

    fn structure(&mut self, start: usize, name: Option<String>) -> Result<Value> {
        if !self.consume(TokenKind::LeftParen) {
            return Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message(format!("Expected `(`, found `{}`", self.peek().text)),
                )
                .with_label(
                    Label::new(start..self.peek().span.start).with_message("Struct begins here"),
                )
                .with_note("Expected `(` at start of struct"));
        }

        let mut fields = IndexMap::default();

        loop {
            let field_name = self.require(TokenKind::Ident)?.text.clone();
            self.require(TokenKind::Colon)?;
            let value = self.value();
            fields.insert(field_name, value);
            if !self.consume(TokenKind::Comma) {
                break;
            }
            if self.peek().kind == TokenKind::RightParen {
                break;
            }
        }
        if !self.consume(TokenKind::RightParen) {
            return Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message(format!("Expected `)`, found `{}`", self.peek().text)),
                )
                .with_label(
                    Label::new(start..self.peek().span.start).with_message("Struct begins here"),
                )
                .with_note("Expected `)` at end of struct"));
        }

        Ok(Value::Struct(Struct { name, fields }))
    }

    fn tuple(&mut self, start: usize) -> Result<Value> {
        if !self.consume(TokenKind::LeftParen) {
            return Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message(format!("Expected `(`, found `{}`", self.peek().text)),
                )
                .with_label(
                    Label::new(start..self.peek().span.start).with_message("Tuple begins here"),
                )
                .with_note("Expected `(` at start of tuple"));
        }
        let mut values = Vec::new();
        loop {
            if self.peek().kind == TokenKind::RightParen {
                break;
            }
            values.push(self.value());
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }

        self.require(TokenKind::RightParen)?;

        if values.is_empty() {
            Ok(Value::Unit)
        } else {
            Ok(Value::Tuple(values))
        }
    }

    fn map(&mut self) -> Result<Value> {
        self.require(TokenKind::LeftBrace)?;
        let mut fields = IndexMap::default();
        loop {
            let key = self.value();
            self.require(TokenKind::Colon)?;
            let value = self.value();
            fields.insert(key, value);
            if !self.consume(TokenKind::Comma) {
                break;
            }
            if self.peek().kind == TokenKind::RightBrace {
                break;
            }
        }

        if !self.consume(TokenKind::RightBrace) {
            return Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message(format!("Expected `}}`, found `{}`", self.peek().text)),
                )
                .with_label(
                    Label::new(self.pos()..self.peek().span.start).with_message("Map begins here"),
                )
                .with_note("Expected `}` at end of map"));
        }

        Ok(Value::Map(Map(fields)))
    }

    fn seq(&mut self) -> Result<Value> {
        self.require(TokenKind::LeftBracket)?;

        let mut values = Vec::new();

        loop {
            if self.peek().kind == TokenKind::RightBracket {
                break;
            }
            values.push(self.value());
            if !self.consume(TokenKind::Comma) {
                // TODO(clemens): recover from missing comma
                break;
            }
        }

        if !self.consume(TokenKind::RightBracket) {
            return Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message(format!("Expected `]`, found `{}`", self.peek().text)),
                )
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end)
                        .with_message("List begins here"),
                )
                .with_note("Expected `]` at end of list"));
        }

        Ok(Value::Seq(values))
    }

    fn check2(&self, kind: TokenKind) -> bool {
        self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == kind
    }

    fn check3(&self, kind: TokenKind) -> bool {
        self.current + 2 < self.tokens.len() && self.tokens[self.current + 2].kind == kind
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token: TokenKind) -> bool {
        if self.peek().kind == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn require(&mut self, token: TokenKind) -> Result<&Token> {
        if self.peek().kind == token {
            Ok(self.advance())
        } else {
            Err(Report::build(ReportKind::Error, (), self.peek().span.start)
                .with_message("Unexpected token".to_string())
                .with_label(
                    Label::new(self.peek().span.start..self.peek().span.end).with_message(format!(
                        "Expected {}, found {}",
                        token,
                        self.peek().kind
                    )),
                ))
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn pos(&self) -> usize {
        self.peek().span.start
    }
}
