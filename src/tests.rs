use std::fs::File;
use std::io::Write;

use ariadne::Config;
use indexmap::indexmap;

use crate::load;
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

static UNIT_STRUCT: &str = r#"
Config(
    version: 1,
    foo: bar
)
"#;

#[test]
fn test_unit_struct() {
    let expected = Value::Struct(Struct {
        prototype: None,
        name: Some("Config".to_string()),
        fields: indexmap! {"version".to_string() => Value::Number(Number::from(1)), "foo".to_string() => Value::Tuple(Some("bar".to_string()), vec![])},
    });

    test_parse(UNIT_STRUCT, expected);
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
 2 │ Config(
   · │ 
   · ╰─ Struct begins here
 5 │ (
   · ┬  
   · ╰── Expected `)`, found `(`
   · 
   · Note: Expected `)` at end of struct
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
            "tuple".to_string() => Value::Tuple(None, vec![
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
 2 │ [
   · │ 
   · ╰─ List begins here
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
    test_parse_with_includes(INCLUDE, expected);
}

static LARGE: &str = r#"
XpV0(
    project: "dcc",
    containers: {
        "trainer": (
            command: ["python", "main.py"],
            env_secrets: {
                "WANDB_API_KEY": "wandb-api-key",
            },
            replicas: 1,
            gpu: 1,
            gpu_mem: "5GB",
            volumes: {
                "/mnt/a/Dropbox/artifacts/xprun": "/mnt/xprun",
            },
            build: [
                From("nvcr.io/nvidia/pytorch:21.03-py3"),

                // install rust toolchain
                Run("apt-get update"),
                Run("apt-get install curl build-essential --yes"),
                Run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"),
                Env("PATH", "/root/.cargo/bin:${PATH}"),
                Run("pip install --upgrade pip"),
                Run("pip install maturin"),

                // build xprun from source
                Repo(url: "git@github.com:cswinter/xprun.git", rev: "eb59b24", cd: true),
                Run("maturin build --cargo-extra-args=--features=python"),
                Run("pip install target/wheels/xprun-0.1.0-cp38-cp38-manylinux_2_27_x86_64.whl"),

                // build pyron from source
                Repo(url: "git@github.com:cswinter/pyron.git", rev: "23825de", cd: true),
                Run("maturin build"),
                Run("pip install target/wheels/pyron-0.1.0-cp38-cp38-manylinux_2_24_x86_64.whl"),

                Repo(path: "requirements.txt", cd: true, rm: true),
                Run("pip install -r requirements.txt"),

                Repo(url: "git@github.com:cswinter/hyperstate.git", rev: "77893bf", cd: true),
                Run("pip install -e ."),

                Repo(cd: true),
            ],
        ),
        "codecraftserver": (
            command: ["server-0.1.0-SNAPSHOT/bin/server", "-Dplay.http.secret.key=ad31779d4ee49d5ad5162bf1429c32e2e9933f3b"],
            cpu: 4,
            cpu_mem: "20GiB",
            tty: true,
            env: {
                "SBT_OPTS": "-Xmx10G",
            },
            build: [
                From("hseeberger/scala-sbt:8u222_1.3.5_2.13.1"),

                // build fixed versions of CodeCraftGame and CodeCraftServer as a straightforward way to download sbt 0.13.16 and populate dependency cache
                Repo(url: "https://github.com/cswinter/CodeCraftGame.git", rev: "92304eb", cd: true, rm: true),
                Run("sbt publishLocal"),
                Repo(url: "https://github.com/cswinter/CodeCraftServer.git", rev: "df76892", cd: true, rm: true),
                Run("sbt compile"),

                // build CodeCraftGame and CodeCraftServer from source
                Repo(url: "https://github.com/cswinter/CodeCraftGame.git", rev: "edc5a9f2", cd: true),
                Run("sbt publishLocal"),
                Repo(url: "https://github.com/cswinter/CodeCraftServer.git", rev: "302a379", cd: true),
                Run("sbt dist"),
                Run("unzip server/target/universal/server-0.1.0-SNAPSHOT.zip"),
            ],
        ),
    }
)
"#;

#[test]
fn test_large() {
    let large_expected = Value::Struct(Struct {
        prototype: None,
        name: Some("XpV0".to_string()),
        fields: indexmap! {"project".to_string() => Value::String("dcc".to_string()), "containers".to_string() => Value::Map(Map(indexmap!{Value::String("trainer".to_string()) => Value::Struct(Struct{prototype:None, name:None, fields: indexmap!{"command".to_string() => Value::Seq(vec![Value::String("python".to_string()), Value::String("main.py".to_string())]), "env_secrets".to_string() => Value::Map(Map(indexmap!{Value::String("WANDB_API_KEY".to_string()) => Value::String("wandb-api-key".to_string())})), "replicas".to_string() => Value::Number(Number::from(1)), "gpu".to_string() => Value::Number(Number::from(1)), "gpu_mem".to_string() => Value::String("5GB".to_string()), "volumes".to_string() => Value::Map(Map(indexmap!{Value::String("/mnt/a/Dropbox/artifacts/xprun".to_string()) => Value::String("/mnt/xprun".to_string())})), "build".to_string() => Value::Seq(vec![Value::Tuple(Some("From".to_string()), vec![Value::String("nvcr.io/nvidia/pytorch:21.03-py3".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("apt-get update".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("apt-get install curl build-essential --yes".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y".to_string())]), Value::Tuple(Some("Env".to_string()), vec![Value::String("PATH".to_string()), Value::String("/root/.cargo/bin:${PATH}".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install --upgrade pip".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install maturin".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("git@github.com:cswinter/xprun.git".to_string()), "rev".to_string() => Value::String("eb59b24".to_string()), "cd".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("maturin build --cargo-extra-args=--features=python".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install target/wheels/xprun-0.1.0-cp38-cp38-manylinux_2_27_x86_64.whl".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("git@github.com:cswinter/pyron.git".to_string()), "rev".to_string() => Value::String("23825de".to_string()), "cd".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("maturin build".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install target/wheels/pyron-0.1.0-cp38-cp38-manylinux_2_24_x86_64.whl".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"path".to_string() => Value::String("requirements.txt".to_string()), "cd".to_string() => Value::Bool(true), "rm".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install -r requirements.txt".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("git@github.com:cswinter/hyperstate.git".to_string()), "rev".to_string() => Value::String("77893bf".to_string()), "cd".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("pip install -e .".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"cd".to_string() => Value::Bool(true)} })])} }), Value::String("codecraftserver".to_string()) => Value::Struct(Struct{prototype:None, name:None, fields: indexmap!{"command".to_string() => Value::Seq(vec![Value::String("server-0.1.0-SNAPSHOT/bin/server".to_string()), Value::String("-Dplay.http.secret.key=ad31779d4ee49d5ad5162bf1429c32e2e9933f3b".to_string())]), "cpu".to_string() => Value::Number(Number::from(4)), "cpu_mem".to_string() => Value::String("20GiB".to_string()), "tty".to_string() => Value::Bool(true), "env".to_string() => Value::Map(Map(indexmap!{Value::String("SBT_OPTS".to_string()) => Value::String("-Xmx10G".to_string())})), "build".to_string() => Value::Seq(vec![Value::Tuple(Some("From".to_string()), vec![Value::String("hseeberger/scala-sbt:8u222_1.3.5_2.13.1".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("https://github.com/cswinter/CodeCraftGame.git".to_string()), "rev".to_string() => Value::String("92304eb".to_string()), "cd".to_string() => Value::Bool(true), "rm".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("sbt publishLocal".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("https://github.com/cswinter/CodeCraftServer.git".to_string()), "rev".to_string() => Value::String("df76892".to_string()), "cd".to_string() => Value::Bool(true), "rm".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("sbt compile".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("https://github.com/cswinter/CodeCraftGame.git".to_string()), "rev".to_string() => Value::String("edc5a9f2".to_string()), "cd".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("sbt publishLocal".to_string())]), Value::Struct(Struct{prototype:None, name:Some("Repo".to_string()), fields: indexmap!{"url".to_string() => Value::String("https://github.com/cswinter/CodeCraftServer.git".to_string()), "rev".to_string() => Value::String("302a379".to_string()), "cd".to_string() => Value::Bool(true)} }), Value::Tuple(Some("Run".to_string()), vec![Value::String("sbt dist".to_string())]), Value::Tuple(Some("Run".to_string()), vec![Value::String("unzip server/target/universal/server-0.1.0-SNAPSHOT.zip".to_string())])])} })}))},
    });
    test_parse(LARGE, large_expected);
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
        Value::Tuple(
            Some("NewType".to_string()),
            vec![Value::Number(Number::Integer(42))],
        ),
    );
    test_parse(
        "(33)",
        Value::Tuple(None, vec![Value::Number(Number::Integer(33))]),
    );
    test_parse(
        "TupleStruct(2,5,)",
        Value::Tuple(
            Some("TupleStruct".to_string()),
            vec![
                Value::Number(Number::Integer(2)),
                Value::Number(Number::Integer(5)),
            ],
        ),
    );
}

fn expect_error(input: &str, error: &str) {
    let parser = Parser::new(input, "<unknown>");
    let (val, errors) = parser.parse();
    if errors.is_empty() {
        panic!("Expected error, got {:?}", val);
    } else {
        let mut first = true;
        for rb in errors {
            let report = rb.with_config(Config::default().with_color(false)).finish();
            let mut err = vec![];
            report
                .write(
                    ariadne::sources(vec![("<unknown>".to_string(), input.to_string())]),
                    &mut err,
                )
                .unwrap();
            let err = String::from_utf8(err).unwrap();
            if first {
                if err != error {
                    report
                        .print(ariadne::sources(vec![(
                            "<unknown>".to_string(),
                            input.to_string(),
                        )]))
                        .unwrap();
                }
                assert_eq!(err, error);
                first = false;
            }
        }
    }
}

fn test_parse(input: &str, expected: Value) {
    let parser = Parser::new(input, "<unknown>");
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();
    if !errors.is_empty() {
        for error in errors {
            error
                .finish()
                .print(ariadne::sources(vec![(
                    "<unknown>".to_string(),
                    input.to_string(),
                )]))
                .unwrap();
        }
        panic!("Expected no errors");
    }
    if val != expected {
        println!("{}", val.fmt_as_rust());
    }
    assert_eq!(val, expected);
}

fn test_parse_with_includes(input: &str, expected: Value) {
    let parser = Parser::new(input, "<unknown>");
    let _tokens = parser.tokens.clone();
    let (val, errors) = parser.parse();
    if !errors.is_empty() {
        // println!("{:#?}", _tokens);
        for error in errors {
            error
                .finish()
                .print(ariadne::sources(vec![(
                    "<unknown>".to_string(),
                    input.to_string(),
                )]))
                .unwrap();
        }
        panic!("Expected no errors");
    }
    assert_eq!(val, expected);
}

static ROOT_CONFIG: &str = r#"
GoblinWizard(
    #prototype("goblin.ron"),
    name: "Goblin Wizard",
    spells: #include("spells.ron"),
)
"#;

static GOBLIN: &str = r#"
Goblin(
    minHealth: 10,
    maxHealth: 20,
    resists: [
        "fire",
        "cold",
    ],
    weaknesses: [
        "lightning",
        "poison",
    ],
)
"#;

static SPELLS: &str = r#"
[
    Spell(
        name: "Fireball",
        damage: 10,
        manaCost: 5,
    ),
    Spell(
        name: "Lightning Bolt",
        damage: 15,
        manaCost: 10,
    ),
]
"#;

#[test]
fn test_resolve() {
    // Create temporary files
    let tmp_dir = tempdir::TempDir::new("root").unwrap();
    // Write all the files
    let mut tmp_file = tmp_dir.path().join("config.ron");
    let mut file = File::create(&tmp_file).unwrap();
    file.write_all(ROOT_CONFIG.as_bytes()).unwrap();
    tmp_file = tmp_dir.path().join("goblin.ron");
    file = File::create(&tmp_file).unwrap();
    file.write_all(GOBLIN.as_bytes()).unwrap();
    tmp_file = tmp_dir.path().join("spells.ron");
    file = File::create(&tmp_file).unwrap();
    file.write_all(SPELLS.as_bytes()).unwrap();

    let value = load(tmp_dir.path().join("config.ron")).unwrap().value;
    let expected = Value::Struct(Struct {
        prototype: None,
        name: Some("GoblinWizard".to_string()),
        fields: indexmap! {"name".to_string() => Value::String("Goblin Wizard".to_string()), "spells".to_string() => Value::Seq(vec![Value::Struct(Struct{prototype:None, name:Some("Spell".to_string()), fields: indexmap!{"name".to_string() => Value::String("Fireball".to_string()), "damage".to_string() => Value::Number(Number::from(10)), "manaCost".to_string() => Value::Number(Number::from(5))} }), Value::Struct(Struct{prototype:None, name:Some("Spell".to_string()), fields: indexmap!{"name".to_string() => Value::String("Lightning Bolt".to_string()), "damage".to_string() => Value::Number(Number::from(15)), "manaCost".to_string() => Value::Number(Number::from(10))} })]), "minHealth".to_string() => Value::Number(Number::from(10)), "maxHealth".to_string() => Value::Number(Number::from(20)), "resists".to_string() => Value::Seq(vec![Value::String("fire".to_string()), Value::String("cold".to_string())]), "weaknesses".to_string() => Value::Seq(vec![Value::String("lightning".to_string()), Value::String("poison".to_string())])},
    });
    if value != expected {
        println!("{}", value.fmt_as_rust());
    }
    assert_eq!(value, expected)
}
