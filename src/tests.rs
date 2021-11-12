use ariadne::{Config, Source};
use indexmap::indexmap;

use crate::value::{Float, Map, Number, Struct, Value};
use crate::Parser;

static SIMPLE_STRUCT: &str = r#"
Config(
    version: 1,
)
"#;

#[test]
fn test_simple_struct() {
    let expected = Value::Struct(Struct {
        name: Some("Config".to_string()),
        fields: indexmap! {"version".to_string() => Value::Number(Number::Integer(1))},
    });
    test_parse(SIMPLE_STRUCT, expected);
}

static INVALID_STRUCT: &str = r#"
Config(
    version: 1,
    foo: bar
)
"#;

static INVALID_STRUCT_ERROR: &str = r#"Error: Unexpected token `)`
   ╭─[<unknown>:5:1]
   │
 4 │     foo: bar
   ·          ──┬─  
   ·            ╰─── Struct begins here
 5 │ )
   · ┬  
   · ╰── Expected `(`, found `)`
   · 
   · Note: Expected `(` at start of struct
───╯
"#;

#[test]
fn test_invalid_struct() {
    expect_error(INVALID_STRUCT, INVALID_STRUCT_ERROR);
}

static INVALID_STRUCT_2: &str = r#"
Config(
    version: 1,
    foo: 4
(
"#;

static INVALID_STRUCT_2_ERROR: &str = r#"Error: Unexpected token `(`
   ╭─[<unknown>:5:1]
   │
 2 │ ╭─▶ Config(
 4 │ ├─▶     foo: 4
   · │                
   · ╰──────────────── Struct begins here
 5 │     (
   ·     ┬  
   ·     ╰── Expected `)`, found `(`
   · │   
   · │   Note: Expected `)` at end of struct
───╯
"#;

#[test]
fn test_invalid_struct_2() {
    expect_error(INVALID_STRUCT_2, INVALID_STRUCT_2_ERROR);
}

static STRUCT_WITH_ALL_TYPES: &str = r#"
Config(
    int: 1,
    float: 1.0,
    bool: true,
    string: "foo",
    list: [1, 2, 3],
    map: {foo: "bar"},
    tuple: (1, 2, 3),
    empty: (),
    none: None,
)"#;

#[test]
fn test_struct_with_all_types() {
    let expected = Value::Struct(Struct {
        name: Some("Config".to_string()),
        fields: indexmap! {
            "int".to_string() => Value::Number(Number::from(1)),
            "float".to_string() => Value::Number(Number::from(1.0)),
            "bool".to_string() => Value::Bool(true),
            "string".to_string() => Value::String("foo".to_string()),
            "list".to_string() => Value::Seq(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(3)),
            ]),
            "map".to_string() => Value::Map(Map(indexmap! {
                Value::String("foo".to_string()) => Value::String("bar".to_string()),
            })),
            "tuple".to_string() => Value::Tuple(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(3)),
            ]),
            "empty".to_string() => Value::Tuple(vec![]),
            "none".to_string() => Value::Option(None),
        },
    });
    test_parse(STRUCT_WITH_ALL_TYPES, expected);
}

fn expect_error(input: &str, error: &str) {
    let parser = Parser::new(input, false);
    let (val, errors) = parser.parse();
    if errors.is_empty() {
        panic!("Expected error, got {:?}", val);
    } else {
        for rb in errors {
            let report = rb.with_config(Config::default().with_color(false)).finish();
            let mut err = vec![];
            report.write(Source::from(input), &mut err).unwrap();
            let err = String::from_utf8(err).unwrap();
            if err != error {
                report.print(Source::from(input)).unwrap();
            }
            assert_eq!(err, error);
        }
    }
}

fn test_parse(input: &str, expected: Value) {
    let parser = Parser::new(input, true);
    let (val, errors) = parser.parse();
    if !errors.is_empty() {
        for error in errors {
            error.finish().print(Source::from(input)).unwrap();
        }
        panic!("Expected no errors");
    }
    assert_eq!(val, expected);
}
