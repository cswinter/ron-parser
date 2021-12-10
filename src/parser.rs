use std::ops::Range;

use ariadne::{Label, Report, ReportBuilder, ReportKind};
use indexmap::IndexMap;

use crate::token::{Token, TokenKind};
use crate::value::{Map, Number, Struct, Value};
use crate::Lexer;

type RB = ReportBuilder<(String, Range<usize>)>;
type Result<T> = std::result::Result<T, ReportBuilder<(String, Range<usize>)>>;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    current: usize,
    errors: Vec<ReportBuilder<(String, Range<usize>)>>,
    source_path: String,
}

impl Parser {
    pub fn new(source: &str, source_path: &str) -> Parser {
        let (tokens, errors) = Lexer::new(source, source_path).scan();
        Parser {
            tokens,
            current: 0,
            errors,
            source_path: source_path.to_string(),
        }
    }

    pub fn parse(mut self) -> (Value, Vec<RB>) {
        let value = self.value();
        if !self.is_at_end() {
            self.errors.push(
                Report::build(ReportKind::Error, "asdf".to_string(), self.pos())
                    .with_message("Expected end of input.")
                    .with_label(
                        Label::new(("asdf".to_string(), self.pos()..self.pos())).with_message(""),
                    ),
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
                            Err(self.error()
                                .with_message(format!("Malformed number `{}`", text))
                                .with_label(
                                    self.label()
                                        .with_message(""),
                                )
                                .with_note(format!("Failed to parse number: {}", err)))
                        }
                    },
                }
            }
            TokenKind::String => self.string().map(Value::String),
            TokenKind::Hash => self.include(),
            token => Err(self.error()
                .with_message("Expected one of `\"`, `[`, `{`, `(`, `true`, `false`, `None`, <ident>, <number>")
                .with_label(
                    self.label()
                        .with_message(format!("Unexpected token `{}` at start of value.", token)),
                )
        ),
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
        if (self.check2(TokenKind::Ident) && self.check3(TokenKind::Colon))
            || (self.check2(TokenKind::Hash)
                && self.check3(TokenKind::Ident)
                && self.check_text3("prototype"))
        {
            self.structure(start, name)
        } else {
            self.tuple(start, name)
        }
    }

    fn structure(&mut self, start: usize, name: Option<String>) -> Result<Value> {
        let mut fields = IndexMap::default();
        let mut prototype = None;

        if self.consume(TokenKind::LeftParen) {
            loop {
                if self.consume(TokenKind::Hash) {
                    let text = self.ident()?;
                    if text != "prototype" {
                        return Err(self
                            .error()
                            .with_message(format!("Unexpected token `{}`", self.peek().kind))
                            .with_label(self.label().with_message(format!(
                                "Expected `prototype` after `#` in struct, found `{}`",
                                text
                            ))));
                    }
                    self.require(TokenKind::LeftParen)?;
                    let path = self.string()?;
                    self.require(TokenKind::RightParen)?;
                    prototype = Some(path);
                } else {
                    let field_name = self.require(TokenKind::Ident)?.text.clone();
                    self.require(TokenKind::Colon)?;
                    let value = self.value();
                    fields.insert(field_name, value);
                }
                if !self.consume(TokenKind::Comma) {
                    break;
                }
                if self.peek().kind == TokenKind::RightParen {
                    break;
                }
            }
            if !self.consume(TokenKind::RightParen) {
                return Err(self
                    .error()
                    .with_message(format!("Unexpected token `{}`", self.peek().kind))
                    .with_label(
                        self.label()
                            .with_message(format!("Expected `)`, found `{}`", self.peek().text)),
                    )
                    .with_label(
                        self.label_span(start..start)
                            .with_message("Struct begins here"),
                    )
                    .with_note("Expected `)` at end of struct"));
            }
        }

        Ok(Value::Struct(Struct {
            name,
            fields,
            prototype,
        }))
    }

    fn tuple(&mut self, _start: usize, name: Option<String>) -> Result<Value> {
        let mut values = Vec::new();
        if self.consume(TokenKind::LeftParen) {
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
        }

        if values.is_empty() && name.is_none() {
            Ok(Value::Unit)
        } else {
            Ok(Value::Tuple(name, values))
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
            return Err(self
                .error()
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    self.label()
                        .with_message(format!("Expected `}}`, found `{}`", self.peek().text)),
                )
                .with_label(
                    self.label_span(self.pos()..self.pos())
                        .with_message("Map begins here"),
                )
                .with_note("Expected `}` at end of map"));
        }

        Ok(Value::Map(Map(fields)))
    }

    fn seq(&mut self) -> Result<Value> {
        let start = self.pos();
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
            return Err(self
                .error()
                .with_message(format!("Unexpected token `{}`", self.peek().kind))
                .with_label(
                    self.label()
                        .with_message(format!("Expected `]`, found `{}`", self.peek().text)),
                )
                .with_label(
                    self.label_span(start..start)
                        .with_message("List begins here"),
                )
                .with_note("Expected `]` at end of list"));
        }

        Ok(Value::Seq(values))
    }

    fn include(&mut self) -> Result<Value> {
        let start = self.pos();
        self.require(TokenKind::Hash)?;
        match self.ident()?.as_ref() {
            "include" => {
                self.require(TokenKind::LeftParen)?;
                let path = self.string()?;
                self.require(TokenKind::RightParen)?;
                Ok(Value::Include(path))
            }
            "prototype" => Err(self
                .error()
                .with_message("Unexpected #prototype directive")
                .with_label(self.label_span(start..self.peek().span.end).with_message(
                    "Expected value but found `#prototype`. Only structs can have prototypes.",
                ))),
            ident => Err(self
                .error()
                .with_message(format!(
                    "Unknown directive `#{}`. Valid directives are `include` and `prototype`.",
                    ident
                ))
                .with_label(self.label())),
        }
    }

    fn string(&mut self) -> Result<String> {
        // TODO: unicode escapes, 7bit character codes
        self.consume(TokenKind::String);
        let mut string = String::new();
        let mut escaped = false;
        let text = self.previous().text.clone();
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
                                self.source_path.to_string(),
                                self.peek().span.start + i - 1,
                            )
                            .with_message(format!("unknown character escape: `\\{}`", char))
                            .with_label(
                                self.label_span(
                                    self.peek().span.start + i - 1..self.peek().span.start + i,
                                )
                                .with_message(""),
                            )
                            .with_note(
                                "Valid escape sequences are: `\\n`, `\\r`, `\\t`, `\\\"`, `\\0`",
                            ),
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
        Ok(string)
    }

    fn check2(&self, kind: TokenKind) -> bool {
        self.current + 1 < self.tokens.len() && self.tokens[self.current + 1].kind == kind
    }

    fn check3(&self, kind: TokenKind) -> bool {
        self.current + 2 < self.tokens.len() && self.tokens[self.current + 2].kind == kind
    }

    fn check_text3(&self, text: &str) -> bool {
        self.current + 2 < self.tokens.len() && self.tokens[self.current + 2].text == text
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
            Err(self
                .error()
                .with_message("Unexpected token".to_string())
                .with_label(self.label().with_message(format!(
                    "Expected {}, found {}",
                    token,
                    self.peek().kind
                ))))
        }
    }

    fn ident(&mut self) -> Result<String> {
        if self.peek().kind == TokenKind::Ident {
            Ok(self.advance().text.clone())
        } else {
            Err(self
                .error()
                .with_message("Unexpected token".to_string())
                .with_label(
                    self.label()
                        .with_message(format!("Expected identifier, found {}", self.peek().kind,)),
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

    fn report(&self, kind: ReportKind) -> ReportBuilder<(String, Range<usize>)> {
        Report::build(kind, self.source_path.to_string(), self.pos())
    }

    fn error(&self) -> ReportBuilder<(String, Range<usize>)> {
        self.report(ReportKind::Error)
    }

    fn label_span(&self, span: Range<usize>) -> Label<(String, Range<usize>)> {
        Label::new((self.source_path.to_string(), span))
    }

    fn label(&self) -> Label<(String, Range<usize>)> {
        Label::new((
            self.source_path.to_string(),
            self.peek().span.start..self.peek().span.end,
        ))
    }
}
