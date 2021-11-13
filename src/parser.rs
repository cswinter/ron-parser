use std::ops::Range;

use ariadne::{Label, Report, ReportBuilder, ReportKind};
use indexmap::IndexMap;

use crate::token::{Span, Token, TokenKind};
use crate::value::{Number, Struct, Value};
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
        //(ASTNode, Vec<CompileError>) {
        let value = self.value();
        if !self.is_at_end() {
            //self.errors.push(CompileError::expeced_eof(self.peek()));
        }
        (value, self.errors)
    }

    fn value(&mut self) -> Value {
        let val = match self.peek().kind {
            TokenKind::Ident => self.structure(),
            TokenKind::LeftParen => self.struct_or_tuple(),
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
                // TODO: handle escapes
                let text = self.advance().text.clone();
                Ok(Value::String(text))
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

    fn structure(&mut self) -> Result<Value> {
        let start = self.pos();
        let name = if self.peek().kind == TokenKind::Ident {
            Some(self.advance().text.clone())
        } else {
            None
        };

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

    fn struct_or_tuple(&self) -> Result<Value> {
        todo!()
    }

    fn map(&self) -> Result<Value> {
        todo!()
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

    fn name(&mut self) -> Result<String> {
        Ok(self.require(TokenKind::Ident)?.text.to_string())
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check2(&self, kind: TokenKind) -> bool {
        self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == kind
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

    fn consume_one_of(&mut self, tokens: &[TokenKind]) -> bool {
        if tokens.contains(&self.peek().kind) {
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

    fn require_one_of(&mut self, tokens: &[TokenKind]) -> Result<&Token> {
        if tokens.contains(&self.peek().kind) {
            Ok(self.advance())
        } else {
            Err(
                Report::build(ReportKind::Error, (), self.peek().span.start).with_message(format!(
                    "Expected one of {} but found {}",
                    tokens
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    self.peek().kind
                )),
            )
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

    /*fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.advance().kind {
                TokenKind::Semi | TokenKind::Newline => return,
                _ => {}
            }
        }
    }*/

    fn pos(&self) -> usize {
        self.peek().span.start
    }

    fn span_from(&self, start: usize) -> Span {
        Span {
            start,
            end: self.previous().span.end,
        }
    }
}
