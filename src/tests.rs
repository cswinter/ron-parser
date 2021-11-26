use ariadne::{Config, Source};
use indexmap::indexmap;

use crate::parser::Parser;
use crate::value::{Map, Number, Struct, Value};

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
        prototype: None,
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
   ·            ╰─── Tuple begins here
 5 │ )
   · ┬  
   · ╰── Expected `(`, found `)`
   · 
   · Note: Expected `(` at start of tuple
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
    /*
    multiline
    comment //
     */
    map: {Struct(a: "14", c: [(), 3, true]): "bar"},
    tuple: (1, 2, 3),
    // comment
    empty: /* comment */ (),
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
                Value::Struct(Struct {
                    name: Some("Struct".to_string()),
                    fields: indexmap! {
                        "a".to_string() => Value::String("14".to_string()),
                        "c".to_string() => Value::Seq(vec![
                            Value::Unit,
                            Value::Number(Number::Integer(3)),
                            Value::Bool(true),
                        ]),
                    },
                    prototype:None,
                }) => Value::String("bar".to_string()),
            })),
            "tuple".to_string() => Value::Tuple(vec![
                Value::Number(Number::Integer(1)),
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(3)),
            ]),
            "empty".to_string() => Value::Unit,
            "none".to_string() => Value::Option(None),
        },
        prototype: None,
    });
    test_parse(STRUCT_WITH_ALL_TYPES, expected);
}

static STRING_ESCAPES: &str = r#"
[
    "foo\n
    bar",
    "foo\r\"\\n\\b\\\\ar\"\"\"\\",
]
"#;

#[test]
fn test_string_escapes() {
    let expected = Value::Seq(vec![
        Value::String("foo\n\n    bar".to_string()),
        Value::String("foo\r\"\\n\\b\\\\ar\"\"\"\\".to_string()),
    ]);
    test_parse(STRING_ESCAPES, expected);
}

static MISSING_CLOSING_BRACKET: &str = r#"
[
    "foo
    bar"
"#;

static MISSING_CLOSING_BRACKET_ERROR: &str = r#"Error: Unexpected token `<EOF>`
   ╭─[<unknown>:4:10]
   │
   · 
   · Note: Expected `]` at end of list
───╯
"#;

#[test]
fn test_missing_closing_bracket() {
    expect_error(MISSING_CLOSING_BRACKET, MISSING_CLOSING_BRACKET_ERROR);
}

static INCLUDE: &str = r#"
GoblinWizard(
    #prototype("goblin.ron"),
    name: "Goblin Wizard",
    spells: #include("spells.ron"),
)
"#;

#[test]
fn test_include() {
    let expected = Value::Struct(Struct {
        name: Some("GoblinWizard".to_string()),
        prototype: Some("goblin.ron".to_string()),
        fields: indexmap! {
            "name".to_string() => Value::String("Goblin Wizard".to_string()),
            "spells".to_string() => Value::Include("spells.ron".to_string()),
        },
    });
    test_parse_with_incudes(INCLUDE, expected);
}

#[test]
fn test_struct() {
    test_parse(
        "MyStruct(x:4,y:7,)",
        Value::Struct(Struct {
            name: Some("MyStruct".to_string()),
            fields: indexmap! {
                "x".to_string() => Value::Number(Number::Integer(4)),
                "y".to_string() => Value::Number(Number::Integer(7)),
            },
            prototype: None,
        }),
    );
    test_parse(
        "(x:4,y:7)",
        Value::Struct(Struct {
            name: None,
            fields: indexmap! {
                "x".to_string() => Value::Number(Number::Integer(4)),
                "y".to_string() => Value::Number(Number::Integer(7)),
            },
            prototype: None,
        }),
    );
    test_parse(
        "NewType(42)",
        Value::Tuple(vec![Value::Number(Number::Integer(42))]),
    );
    test_parse(
        "(33)",
        Value::Tuple(vec![Value::Number(Number::Integer(33))]),
    );
    test_parse(
        "TupleStruct(2,5,)",
        Value::Tuple(vec![
            Value::Number(Number::Integer(2)),
            Value::Number(Number::Integer(5)),
        ]),
    );
}

fn expect_error(input: &str, error: &str) {
    let parser = Parser::new(input);
    let (val, errors) = parser.parse();
    if errors.is_empty() {
        panic!("Expected error, got {:?}", val);
    } else {
        let mut first = true;
        for rb in errors {
            let report = rb.with_config(Config::default().with_color(false)).finish();
            let mut err = vec![];
            report.write(Source::from(input), &mut err).unwrap();
            let err = String::from_utf8(err).unwrap();
            if first {
                if err != error {
                    report.print(Source::from(input)).unwrap();
                }
                assert_eq!(err, error);
                first = false;
            }
        }
    }
}

fn test_parse(input: &str, expected: Value) {
    let parser = Parser::new(input);
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();
    if !errors.is_empty() {
        // println!("{:#?}", _tokens);
        for error in errors {
            error.finish().print(Source::from(input)).unwrap();
        }
        panic!("Expected no errors");
    }
    assert_eq!(val, expected);
}

fn test_parse_with_incudes(input: &str, expected: Value) {
    let parser = Parser::new(input);
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();
    if !errors.is_empty() {
        // println!("{:#?}", _tokens);
        for error in errors {
            error.finish().print(Source::from(input)).unwrap();
        }
        panic!("Expected no errors");
    }
    assert_eq!(val, expected);
}
