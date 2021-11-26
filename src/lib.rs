pub mod lexer;
mod parser;
#[cfg(test)]
mod tests;
pub mod token;
pub mod value;

use std::ops::Range;

use ariadne::{Report, Source};
use lexer::Lexer;

pub struct Error {
    pub partial_parse: value::Value,
    errors: Vec<Report<Range<usize>>>,
    source: String,
}

impl Error {
    pub fn emit(&self) {
        for error in &self.errors {
            error.eprint(Source::from(&self.source)).unwrap();
        }
    }
}

pub fn parse(source: &str) -> Result<value::Value, Error> {
    let parser = parser::Parser::new(source);
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();

    if errors.is_empty() {
        Ok(val)
    } else {
        Err(Error {
            partial_parse: val,
            errors: errors
                .into_iter()
                .map(|report_builder| report_builder.finish())
                .collect(),
            source: source.to_string(),
        })
    }
}
