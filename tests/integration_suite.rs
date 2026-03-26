mod integration;
mod support;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use dolang::interpreter::DolangValue;
use dolang::module::ProjectConfig;
use dolang::runtime::RuntimeMode;

use integration::{assert_fixture_stdout, assert_route_string};
use support::{route_by_signature, run_fixture};

#[test]
fn runtime_fixtures_cover_core_semantics() {
    assert_fixture_stdout("fixtures/runtime/variables_consts.dol", "3\n10\n");
    assert_fixture_stdout("fixtures/runtime/type_annotations.dol", "Int\nString\n");
    assert_fixture_stdout("fixtures/runtime/functions_control_flow.dol", "pass\n");
    assert_fixture_stdout("fixtures/runtime/collections_builtins.dol", "3\ntrue\n2\n");
}

#[test]
fn runtime_fixture_rejects_variable_shadowing_existing_constant() {
    let outcome = run_fixture(
        "fixtures/runtime/const_shadowing_error.dol",
        RuntimeMode::Test,
    );

    let error = outcome
        .error
        .expect("constant shadowing fixture should fail");
    assert!(error.contains("constant 'MAX' is already defined"));
}

#[test]
fn module_fixture_runs_as_script_level_integration_test() {
    let outcome = run_fixture("fixtures/modules/relative/main.dol", RuntimeMode::Test);
    assert!(outcome.error.is_none(), "module fixture should execute");
    assert_eq!(outcome.stdout, "42\n");
    assert_eq!(outcome.state.env.get("result"), Some(&DolangValue::Int(42)));
    assert!(outcome.state.env.contains_key("helper"));
}

#[test]
fn http_fixture_covers_route_registration_and_test_mode_execution() {
    let outcome = run_fixture("fixtures/http/echo_route.dol", RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http fixture should execute");
    assert_eq!(outcome.context.routes().len(), 2);
    assert!(route_by_signature(outcome.context.routes(), "GET", "/health").is_some());
    assert_route_string("fixtures/http/echo_route.dol", "GET", "/health", "ok");
}

#[test]
fn module_imports_use_filename_namespace_and_hide_private_functions() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-mod-private-{unique}"));
    fs::create_dir_all(project_dir.join("math")).expect("math dir");
    fs::write(
        project_dir.join("main.dol"),
        "$mod math.util;\n$ result = util.add(40, 2);\n$>> result;\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("math/util.dol"),
        "_$fn helper() -> Int { $# 2; }\n$fn add(a, b) -> Int { $# a + b + helper(); }\n",
    )
    .expect("util");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "module import should succeed");
    assert_eq!(outcome.stdout, "44\n");
    assert!(outcome.state.env.contains_key("util"));

    fs::write(
        project_dir.join("main.dol"),
        "$mod math.util;\n$ value = util.helper();\n",
    )
    .expect("private access main");
    let private_outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    let error = private_outcome
        .error
        .expect("private helper should not be visible");
    assert!(error.contains("'helper' not found"));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn wildcard_imports_only_direct_child_modules() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-mod-wildcard-{unique}"));
    fs::create_dir_all(project_dir.join("math/internal")).expect("math dirs");
    fs::write(
        project_dir.join("main.dol"),
        "$mod math.*;\n$ a = util.answer();\n$ b = calc.answer();\n$>> a;\n$>> b;\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("math/util.dol"),
        "$fn answer() -> Int { $# 7; }\n",
    )
    .expect("util");
    fs::write(
        project_dir.join("math/calc.dol"),
        "$fn answer() -> Int { $# 9; }\n",
    )
    .expect("calc");
    fs::write(
        project_dir.join("math/internal/hidden.dol"),
        "$fn answer() -> Int { $# 99; }\n",
    )
    .expect("hidden");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "wildcard import should succeed");
    assert_eq!(outcome.stdout, "7\n9\n");
    assert!(outcome.state.env.contains_key("util"));
    assert!(outcome.state.env.contains_key("calc"));
    assert!(!outcome.state.env.contains_key("hidden"));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn module_import_conflicts_fail_fast() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-mod-conflict-{unique}"));
    fs::create_dir_all(project_dir.join("foo")).expect("foo dir");
    fs::create_dir_all(project_dir.join("bar")).expect("bar dir");
    fs::write(
        project_dir.join("main.dol"),
        "$mod foo.tool;\n$mod bar.tool;\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("foo/tool.dol"),
        "$fn a() -> Int { $# 1; }\n",
    )
    .expect("foo tool");
    fs::write(
        project_dir.join("bar/tool.dol"),
        "$fn b() -> Int { $# 2; }\n",
    )
    .expect("bar tool");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    let error = outcome.error.expect("conflicting namespace should fail");
    assert!(error.contains("module namespace conflict"));
    assert!(error.contains("tool"));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn http_link_extracts_top_level_and_http_block_routes_only() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-link-{unique}"));
    fs::create_dir_all(project_dir.join("routers")).expect("routers dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"routers.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("routers/api.dol"),
        "$fn helper() -> Int { $# 1; }\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n$HTTP(\"/admin\") { $GET(\"/stats\") stats() -> String { $# \"stats\"; } }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");
    assert!(route_by_signature(outcome.context.routes(), "GET", "/v1/health").is_some());
    assert!(route_by_signature(outcome.context.routes(), "GET", "/v1/admin/stats").is_some());

    fs::write(
        project_dir.join("routers/empty.dol"),
        "$fn helper() -> Int { $# 1; }\n",
    )
    .expect("empty router");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"routers.empty\");\n",
    )
    .expect("main");
    let empty_outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    let error = empty_outcome
        .error
        .expect("empty router module should fail");
    assert!(error.contains("does not define any HTTP routes"));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn serve_mode_reads_project_config_from_manifest() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-phase5-config-{unique}"));
    fs::create_dir_all(&project_dir).expect("temp project dir");

    let manifest = r#"
name = "phase5"
version = "0.1.0"
entry = "main.dol"

[env]
APP_MODE = "serve"
"#;
    fs::write(project_dir.join("package.toml"), manifest).expect("manifest");
    fs::write(
        project_dir.join("main.dol"),
        "$ value = $<<CONFIG(\"APP_MODE\");\n$>> value;\n",
    )
    .expect("main");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        outcome.error.is_none(),
        "serve mode config read should succeed"
    );
    assert_eq!(outcome.stdout, "serve\n");
    assert_eq!(
        outcome.state.env.get("value"),
        Some(&DolangValue::Str("serve".to_string()))
    );

    let loaded_config = ProjectConfig::load_from_dir(&project_dir).expect("manifest should parse");
    assert_eq!(loaded_config.get("APP_MODE"), Some("serve"));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}
