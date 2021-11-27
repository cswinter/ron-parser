pub mod lexer;
mod parser;
#[cfg(test)]
mod tests;
pub mod token;
pub mod value;

use std::ops::Range;

use ariadne::Report;
use lexer::Lexer;

pub struct Error {
    pub partial_parse: value::Value,
    errors: Vec<Report<(String, Range<usize>)>>,
    sources: Vec<(String, String)>,
}

impl Error {
    pub fn emit(&self) {
        for error in &self.errors {
            let cache = ariadne::sources(self.sources.clone());
            error.eprint(cache).unwrap();
        }
    }
}

pub fn parse(source: &str, source_name: Option<&str>) -> Result<value::Value, Error> {
    let source_name = source_name.unwrap_or("<unknown>");
    let parser = parser::Parser::new(source, source_name);
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
            sources: vec![(source_name.to_string(), source.to_string())],
        })
    }
}
