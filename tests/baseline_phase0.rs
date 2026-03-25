use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dolang::interpreter::DolangValue;
use dolang::parse;
use dolang::runtime::{
    IntrinsicRegistry, ProgramState, RuntimeContext, RuntimeMode, RuntimePolicy,
    execute_program_with_writer, execute_source_with_writer, intrinsics::ids,
    load_context_and_program,
};

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
        "program should not exit during baseline tests"
    );

    let output = String::from_utf8(output).expect("stdout should be utf-8");
    (state, context, output)
}

fn run_program_error(source: &str) -> String {
    let statements = parse(source).expect("source should parse");
    let mut state = ProgramState::new();
    let mut context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));
    let mut output = Vec::new();

    execute_program_with_writer(&statements, &mut state, &mut context, &mut output)
        .expect_err("program should fail")
        .to_string()
}

#[test]
fn runs_minimal_script_fixture() {
    let source = include_str!("fixtures/runtime/hello.dol");
    let (_state, _context, output) = run_program(source);

    assert_eq!(output, "Hello, Dolang!\n");
}

#[test]
fn tracks_variable_declaration_and_assignment() {
    let source = "$ counter = 1; counter = 2;";
    let (state, _context, _output) = run_program(source);

    assert_eq!(state.env.get("counter"), Some(&DolangValue::Int(2)));
}

#[test]
fn variable_declaration_cannot_shadow_existing_constant() {
    let error = run_program_error("$@ MAX = 300; $ MAX = 30;");

    assert!(error.contains("constant 'MAX' is already defined"));
}

#[test]
fn supports_function_definition_and_call() {
    let source = r#"
$fn add(a, b) -> Int {
    $# a + b;
}

$ result = add(1, 2);
"#;
    let (state, _context, _output) = run_program(source);

    assert_eq!(state.env.get("result"), Some(&DolangValue::Int(3)));
}

#[test]
fn evaluates_expression_precedence() {
    let source = "$ value = 1 + 2 * 3;";
    let (state, _context, _output) = run_program(source);

    assert_eq!(state.env.get("value"), Some(&DolangValue::Int(7)));
}

#[test]
fn registers_http_route_from_fixture() {
    let source = include_str!("fixtures/http/health_route.dol");
    let (_state, context, _output) = run_program(source);

    let routes = context.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].method, "GET");
    assert_eq!(routes[0].path, "/health");
    assert_eq!(routes[0].name, "health");
}

#[test]
fn runtime_context_does_not_leak_routes_between_executions() {
    let source = include_str!("fixtures/http/health_route.dol");

    let (_state_a, context_a, _output_a) = run_program(source);
    let (_state_b, context_b, _output_b) = run_program("$ name = \"isolated\";");

    assert_eq!(context_a.routes().len(), 1);
    assert!(context_b.routes().is_empty());
}

#[test]
fn execute_source_with_writer_uses_explicit_context() {
    let mut state = ProgramState::new();
    let mut context = RuntimeContext::new(RuntimeMode::Repl, PathBuf::from("."));
    let mut output = Vec::new();

    let should_continue = execute_source_with_writer(
        "$ msg = \"hello\"; $>> msg;",
        &mut state,
        &mut context,
        &mut output,
    )
    .expect("source should execute");

    assert!(should_continue);
    assert_eq!(
        state.env.get("msg"),
        Some(&DolangValue::Str("hello".to_string()))
    );
    assert_eq!(String::from_utf8(output).expect("utf-8"), "hello\n");
}

#[test]
fn loader_honors_package_entry_override() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-phase1-{unique}"));
    fs::create_dir_all(&project_dir).expect("temp project dir should be created");

    let package = "name = \"phase1\"\nversion = \"0.1.0\"\nentry = \"app.dol\"\n";
    fs::write(project_dir.join("package.toml"), package).expect("package.toml should be written");
    fs::write(project_dir.join("app.dol"), "$ value = 42;").expect("entry file should be written");

    let (context, program) =
        load_context_and_program(RuntimeMode::Run, &project_dir).expect("program should load");

    assert_eq!(program.path, project_dir.join("app.dol"));
    assert_eq!(
        context.current_file(),
        Some(program.path.to_string_lossy().as_ref())
    );
    assert_eq!(context.project_root(), project_dir.as_path());

    fs::remove_dir_all(&project_dir).expect("temp project dir should be removed");
}

#[test]
fn runtime_context_exposes_intrinsic_registry_for_file_io() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dolang-intrinsic-file-{unique}.txt"));
    let context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));

    context
        .call_intrinsic(
            ids::FS_WRITE_TEXT,
            &[
                DolangValue::Str(path.display().to_string()),
                DolangValue::Str("hello".to_string()),
            ],
        )
        .expect("write intrinsic should succeed");

    let content = context
        .call_intrinsic(
            ids::FS_READ_TEXT,
            &[DolangValue::Str(path.display().to_string())],
        )
        .expect("read intrinsic should succeed");
    assert_eq!(content, DolangValue::Str("hello".to_string()));

    fs::remove_file(path).expect("temp file should be removed");
}

#[test]
fn runtime_policy_can_block_intrinsic_calls() {
    let mut context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));
    let mut policy = RuntimePolicy::allow_all();
    policy.deny(ids::ENV_GET);
    context.set_runtime_policy(policy);

    let error = context
        .call_intrinsic(ids::ENV_GET, &[DolangValue::Str("HOME".to_string())])
        .expect_err("denied intrinsic should fail");
    assert!(error.to_string().contains("denied by runtime policy"));
}

#[test]
fn runtime_context_can_surface_unregistered_intrinsic_error() {
    let mut context = RuntimeContext::new(RuntimeMode::Test, PathBuf::from("."));
    let mut registry = IntrinsicRegistry::new();
    registry.unregister(ids::CONFIG_GET);
    context.set_intrinsic_registry(registry);

    let error = context
        .call_intrinsic(ids::CONFIG_GET, &[DolangValue::Str("APP_ENV".to_string())])
        .expect_err("missing intrinsic should fail");
    assert!(error.to_string().contains("is not registered"));
}

#[test]
fn file_value_methods_keep_working_after_intrinsic_refactor() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dolang-file-methods-{unique}.txt"));
    let source = format!(
        "$ writer = $>>FILE(\"{}\"); writer.content(\"hello\\nworld\");\n\
         $ file = $<<FILE(\"{}\", \"LINES\");\n\
         $ exists = file.exists();\n\
         $ size = file.size();\n\
         $ lines = file.read();\n",
        path.display(),
        path.display()
    );

    let (state, _context, _output) = run_program(&source);
    assert_eq!(state.env.get("exists"), Some(&DolangValue::Bool(true)));
    assert_eq!(state.env.get("size"), Some(&DolangValue::Int(11)));
    assert_eq!(
        state.env.get("lines"),
        Some(&DolangValue::List(vec![
            DolangValue::Str("hello".to_string()),
            DolangValue::Str("world".to_string())
        ]))
    );

    fs::remove_file(path).expect("temp file should be removed");
}
