use std::path::PathBuf;

use dolang::ast::Stmt;
use dolang::interpreter::DolangValue;
use dolang::parse;
use dolang::runtime::{ProgramState, RuntimeContext, RuntimeMode, execute_program_with_writer};

fn run_program(source: &str) -> (ProgramState, RuntimeContext, String) {
    let statements = parse(source).expect("source should parse");
    let mut state = ProgramState::new();
    let mut context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));
    let mut output = Vec::new();

    let should_continue =
        execute_program_with_writer(&statements, &mut state, &mut context, &mut output)
            .expect("program should execute successfully");
    assert!(
        should_continue,
        "program should not exit during Phase 3 tests"
    );

    (
        state,
        context,
        String::from_utf8(output).expect("stdout should be utf-8"),
    )
}

#[test]
fn parser_keeps_statement_families_working_after_split() {
    let source = r#"
$if 1 < 2 {
    $ value = 1;
}

$for $ i = 0; i < 3; i = i + 1 {
    $>> i;
}

$HTTP("/api") {
    $GET("/health") health() -> Int {
        $# 200;
    }
}

$STATIC("public");
"#;

    let statements = parse(source).expect("source should parse");
    assert!(matches!(statements[0], Stmt::If(_)));
    assert!(matches!(statements[1], Stmt::For(_)));
    assert!(matches!(statements[2], Stmt::HttpBlock(_)));
    assert!(matches!(statements[3], Stmt::Static(_)));
}

#[test]
fn exec_and_eval_keep_behavior_after_split() {
    let source = r#"
$ sum = 0;

$for $ i = 0; i < 3; i = i + 1 {
    sum += i;
}

$fn greet(name) -> String {
    $# f"hi {name}";
}

$ msg = greet("dolang");
"#;

    let (state, _context, _output) = run_program(source);
    assert_eq!(state.env.get("sum"), Some(&DolangValue::Int(3)));
    assert_eq!(
        state.env.get("msg"),
        Some(&DolangValue::Str("hi dolang".to_string()))
    );
}

#[test]
fn builtins_method_dispatch_regression_stays_green() {
    let source = r#"
$ upper = "dolang".upper();
$ parts = "a,b,c".split(",");
$ joined = parts.join("-");
$ count = parts.len();
"#;

    let (state, _context, _output) = run_program(source);
    assert_eq!(
        state.env.get("upper"),
        Some(&DolangValue::Str("DOLANG".to_string()))
    );
    assert_eq!(
        state.env.get("joined"),
        Some(&DolangValue::Str("a-b-c".to_string()))
    );
    assert_eq!(state.env.get("count"), Some(&DolangValue::Int(3)));
}
