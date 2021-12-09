pub mod lexer;
mod parser;
#[cfg(test)]
mod tests;
pub mod token;
pub mod value;

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

use ariadne::Report;
use lexer::Lexer;
use value::{Struct, Value};

pub struct Parse {
    pub value: value::Value,
    pub errors: Vec<Report<(String, Range<usize>)>>,
    pub sources: Vec<(String, String)>,
}

impl Parse {
    pub fn emit(&self) {
        for error in &self.errors {
            let cache = ariadne::sources(self.sources.clone());
            error.eprint(cache).unwrap();
        }
    }
}

pub fn parse(source: &str, source_name: Option<&str>) -> Result<value::Value, Parse> {
    let source_name = source_name.unwrap_or("<unknown>");
    let parser = parser::Parser::new(source, source_name);
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();

    if errors.is_empty() {
        Ok(val)
    } else {
        Err(Parse {
            value: val,
            errors: errors
                .into_iter()
                .map(|report_builder| report_builder.finish())
                .collect(),
            sources: vec![(source_name.to_string(), source.to_string())],
        })
    }
}

fn _load<P: AsRef<std::path::Path>>(path: P) -> Result<Parse, std::io::Error> {
    let source = std::fs::read_to_string(path.as_ref())?;
    let source_name = path.as_ref().to_str().unwrap();
    match parse(&source, Some(source_name)) {
        Err(err) => Ok(err),
        Ok(val) => Ok(Parse {
            value: val,
            errors: vec![],
            sources: vec![(source_name.to_string(), source.to_string())],
        }),
    }
}

struct Loader {
    errors: Vec<Report<(String, Range<usize>)>>,
    sources: Vec<(String, String)>,
    resolve_stack: Vec<PathBuf>,
    cache: HashMap<PathBuf, Option<Value>>,
}

impl Loader {
    fn load(&mut self, path: &Path) -> Result<Value, std::io::Error> {
        let path = path.canonicalize()?;
        match self.cache.get(&path) {
            Some(None) => todo!("CYCLE"),
            Some(Some(val)) => return Ok(val.clone()),
            None => {}
        }
        self.resolve_stack.push(path.clone());
        let mut parse = _load(&path)?;
        self.errors.append(&mut parse.errors);
        self.sources.append(&mut parse.sources);
        self.resolve(&mut parse.value, &path)?;
        self.resolve_stack.pop();
        self.cache.insert(path, Some(parse.value.clone()));
        Ok(parse.value)
    }

    fn resolve(&mut self, value: &mut Value, origin: &Path) -> Result<(), std::io::Error> {
        match value {
            Value::Include(path) => *value = self.load(&origin.parent().unwrap().join(path))?,
            Value::Struct(Struct {
                name: _,
                prototype,
                fields,
            }) => {
                if let Some(path) = prototype.as_ref() {
                    let include_path = origin.parent().unwrap().join(path);
                    let include_value = self.load(&include_path)?;
                    match include_value {
                        Value::Struct(include_struct) => {
                            for (name, field) in include_struct.fields.into_iter() {
                                if !fields.contains_key(&name) {
                                    fields.insert(name, field.clone());
                                }
                            }
                        }
                        _ => eprintln!("MUST BE STRUCT {:?}", include_value),
                    }
                    *prototype = None;
                }
                for field in fields.values_mut() {
                    self.resolve(field, origin)?;
                }
            }
            Value::Map(items) => {
                for value in items.0.values_mut() {
                    self.resolve(value, origin)?;
                }
            }
            Value::Seq(values) => {
                for value in values {
                    self.resolve(value, origin)?;
                }
            }
            Value::Bool(_)
            | Value::Char(_)
            | Value::Number(_)
            | Value::Option(_)
            | Value::String(_)
            | Value::Tuple(_)
            | Value::Unit => {}
        }
        Ok(())
    }
}

pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Parse, std::io::Error> {
    let mut loader = Loader {
        errors: vec![],
        sources: vec![],
        resolve_stack: vec![],
        cache: HashMap::new(),
    };
    let value = loader.load(path.as_ref())?;
    Ok(Parse {
        value,
        errors: loader.errors,
        sources: loader.sources,
    })
}
