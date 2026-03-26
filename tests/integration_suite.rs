mod integration;
mod support;

use std::fs;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dolang::interpreter::DolangValue;
use dolang::module::ProjectConfig;
use dolang::parser;
use dolang::runtime::{ProgramState, RuntimeContext, RuntimeMode, execute_program_with_writer};

use integration::{assert_fixture_stdout, assert_route_string};
use support::{
    http_get, http_options, http_request, http_request_with_headers,
    http_request_with_headers_and_body,
    route_by_signature, run_fixture, run_program_at_path, start_live_http_server,
    start_live_http_server_from_context, try_start_live_http_server_from_context,
};

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
fn http_block_headers_propagate_to_child_routes() {
    let outcome = run_fixture("fixtures/http/set_hdr_block.dol", RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http fixture should execute");

    let route = route_by_signature(outcome.context.routes(), "GET", "/api/users")
        .expect("route should exist");

    assert_eq!(
        route.response_headers,
        vec![("X-Frame-Options".to_string(), "DENY".to_string())]
    );
}

#[test]
fn route_headers_override_same_name_block_headers() {
    let outcome = run_fixture("fixtures/http/set_hdr_override.dol", RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http fixture should execute");

    let route = route_by_signature(outcome.context.routes(), "GET", "/api/static")
        .expect("route should exist");

    assert_eq!(
        route.response_headers,
        vec![
            ("Cache-Control".to_string(), "max-age=3600".to_string()),
            ("X-Route".to_string(), "static".to_string()),
        ]
    );
}

#[test]
fn live_http_route_headers_are_emitted() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-route-hdr-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@SET_HDR({ \"X-Route\": \"route\" })\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "live http fixture should boot");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/health");

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-Route"))
            .map(|(_, value)| value.as_str()),
        Some("route")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_block_and_route_headers_merge_and_override() {
    let base_url = start_live_http_server("fixtures/http/set_hdr_override.dol");
    let response = http_get(&base_url, "/api/static");

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Cache-Control"))
            .map(|(_, value)| value.as_str()),
        Some("max-age=3600")
    );
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-Route"))
            .map(|(_, value)| value.as_str()),
        Some("static")
    );
}

#[test]
fn live_http_route_level_multiple_headers_are_emitted() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-route-hdr-multi-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@SET_HDR({ \"X-One\": \"1\", \"X-Two\": \"2\" })\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/health");

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-One"))
            .map(|(_, value)| value.as_str()),
        Some("1")
    );
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-Two"))
            .map(|(_, value)| value.as_str()),
        Some("2")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_block_headers_apply_to_child_routes() {
    let base_url = start_live_http_server("fixtures/http/set_hdr_block.dol");
    let response = http_get(&base_url, "/api/users");

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-Frame-Options"))
            .map(|(_, value)| value.as_str()),
        Some("DENY")
    );
}

#[test]
fn live_http_without_set_hdr_does_not_emit_custom_headers() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-no-hdr-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/health");

    assert_eq!(response.status, 200);
    assert!(
        response
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("X-One")
                && !name.eq_ignore_ascii_case("X-Two")
                && !name.eq_ignore_ascii_case("X-Route")
                && !name.eq_ignore_ascii_case("X-App")),
        "{:?}",
        response.headers
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_skips_invalid_response_headers() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-invalid-hdr-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@SET_HDR({ \"bad header\": \"ignored\", \"X-Valid\": \"visible\" })\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "live http fixture should boot");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/health");

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("X-Valid"))
            .map(|(_, value)| value.as_str()),
        Some("visible")
    );
    assert!(
        response
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("bad header"))
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn linked_module_routes_inherit_parent_block_headers() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-set-hdr-link-{unique}"));
    fs::create_dir_all(project_dir.join("routers")).expect("routers dir");
    fs::write(
        project_dir.join("main.dol"),
        "@SET_HDR({ \"X-Frame-Options\": \"DENY\" })\n$HTTP(\"/api\").link(\"routers.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("routers/api.dol"),
        "$GET(\"/users\") list() -> String { $# \"ok\"; }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");

    let route = route_by_signature(outcome.context.routes(), "GET", "/api/users")
        .expect("route should exist");
    assert_eq!(
        route.response_headers,
        vec![("X-Frame-Options".to_string(), "DENY".to_string())]
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn cors_fixture_propagates_global_block_and_route_configs() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-cors-block-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        support::read_text("fixtures/http/cors_block.dol"),
    )
    .expect("main");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http fixture should execute");

    let public_route = route_by_signature(outcome.context.routes(), "GET", "/api/public")
        .expect("public route should exist");
    let admin_route = route_by_signature(outcome.context.routes(), "GET", "/api/admin")
        .expect("admin route should exist");

    let global_cors = outcome.context.global_cors().expect("global cors");
    assert!(global_cors.allow_all);

    let public_cors = public_route
        .parent_cors
        .as_ref()
        .expect("public route inherited cors");
    assert_eq!(
        public_cors.origins,
        vec!["https://public.example.com".to_string()]
    );
    assert_eq!(
        public_cors.methods,
        vec!["GET".to_string(), "POST".to_string()]
    );
    assert!(!public_cors.allow_all);

    let admin_cors = admin_route.cors.as_ref().expect("admin route cors");
    assert_eq!(
        admin_cors.origins,
        vec!["https://admin.example.com".to_string()]
    );
    assert_eq!(admin_cors.methods, vec!["GET".to_string()]);
    assert!(!admin_cors.allow_all);

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn linked_module_routes_inherit_parent_block_cors() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-cors-link-{unique}"));
    fs::create_dir_all(project_dir.join("routers")).expect("routers dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n@CORS({ origins: [\"https://api.example.com\"], methods: [\"GET\"] })\n$HTTP(\"/api\").link(\"routers.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("routers/api.dol"),
        "$GET(\"/users\") list() -> String { $# \"ok\"; }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");

    let route = route_by_signature(outcome.context.routes(), "GET", "/api/users")
        .expect("route should exist");
    let cors = route
        .parent_cors
        .as_ref()
        .expect("linked route inherited cors");
    assert_eq!(cors.origins, vec!["https://api.example.com".to_string()]);
    assert_eq!(cors.methods, vec!["GET".to_string()]);

    let global_cors = outcome.context.global_cors().expect("global cors");
    assert!(global_cors.allow_all);

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn direct_execution_seeds_global_cors_from_main_statement() {
    let source = r#"
@CORS("*")
$main() {}
"#;
    let statements = parser::parse(source).expect("source should parse");
    let mut context = RuntimeContext::new(RuntimeMode::Test, std::env::temp_dir());
    context.set_current_file(Some("main.dol".to_string()));
    let mut state = ProgramState::new();
    let mut output = Vec::new();

    execute_program_with_writer(&statements, &mut state, &mut context, &mut output)
        .expect("program should execute");

    let global_cors = context.global_cors().expect("global cors");
    assert!(global_cors.allow_all);
}

#[test]
fn clear_routes_preserves_global_cors() {
    let source = r#"
@CORS("*")
$main() {}

$GET("/health") health() -> String {
    $# "ok";
}
"#;
    let statements = parser::parse(source).expect("source should parse");
    let mut context = RuntimeContext::new(RuntimeMode::Test, std::env::temp_dir());
    context.set_current_file(Some("main.dol".to_string()));
    let mut state = ProgramState::new();
    let mut output = Vec::new();

    execute_program_with_writer(&statements, &mut state, &mut context, &mut output)
        .expect("program should execute");
    assert_eq!(context.routes().len(), 1);

    context.clear_routes();

    assert!(context.routes().is_empty());
    assert!(
        context.global_cors().is_some(),
        "global cors should be preserved"
    );
}

#[test]
fn live_http_global_cors_allow_all_handles_preflight() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-global-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_options(
        &base_url,
        "/health",
        &[
            ("Origin", "https://client.example.com"),
            ("Access-Control-Request-Method", "GET"),
        ],
    );

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
            .map(|(_, value)| value.as_str()),
        Some("*")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_route_level_cors_overrides_global_config() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-route-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n@CORS({ origins: [\"https://trusted.example.com\"], methods: [\"GET\"] })\n$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://trusted.example.com")],
    );

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
            .map(|(_, value)| value.as_str()),
        Some("https://trusted.example.com")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_global_cors_whitelist_only_allows_listed_origins() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-whitelist-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"https://allowed.example.com\"], methods: [\"GET\"] })\n$main() {}\n\n$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let allowed = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://allowed.example.com")],
    );
    assert_eq!(
        allowed
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
            .map(|(_, value)| value.as_str()),
        Some("https://allowed.example.com")
    );

    let blocked = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://blocked.example.com")],
    );
    assert!(
        blocked
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("Access-Control-Allow-Origin")),
        "{:?}",
        blocked.headers
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_block_level_cors_override_is_emitted() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-block-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n@CORS({ origins: [\"https://api.example.com\"], methods: [\"GET\"] })\n$HTTP(\"/api\") {\n    $GET(\"/users\") users() -> String { $# \"ok\"; }\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/api/users",
        &[("Origin", "https://api.example.com")],
    );

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
            .map(|(_, value)| value.as_str()),
        Some("https://api.example.com")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_cors_headers_do_not_emit_expose_headers() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-headers-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"https://client.example.com\"], methods: [\"GET\"], headers: [\"Authorization\"] })\n$main() {}\n\n$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let preflight = http_options(
        &base_url,
        "/data",
        &[
            ("Origin", "https://client.example.com"),
            ("Access-Control-Request-Method", "GET"),
            ("Access-Control-Request-Headers", "Authorization"),
        ],
    );

    assert_eq!(preflight.status, 200);
    assert_eq!(
        preflight
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Headers"))
            .map(|(_, value)| value.as_str()),
        Some("authorization")
    );
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://client.example.com")],
    );
    assert_eq!(response.status, 200);
    assert!(
        response
            .headers
            .iter()
            .all(|(name, _)| { !name.eq_ignore_ascii_case("Access-Control-Expose-Headers") }),
        "{:?}",
        response.headers
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn invalid_cors_wildcard_with_credentials_fails_serve_bootstrap() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-invalid-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"*\"], credentials: true })\n$main() {}\n\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "program load should succeed");
    let error = try_start_live_http_server_from_context(outcome.context)
        .expect_err("invalid cors config should fail during serve startup");
    assert!(error.contains("DOL-C002"), "{error}");
    assert!(error.contains("credentials: true"), "{error}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn invalid_cors_negative_max_age_fails_serve_bootstrap() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-max-age-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"https://client.example.com\"], methods: [\"GET\"], max_age: -1 })\n$main() {}\n\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "program load should succeed");
    let error = try_start_live_http_server_from_context(outcome.context)
        .expect_err("invalid cors config should fail during serve startup");
    assert!(error.contains("DOL-C002"), "{error}");
    assert!(error.contains("max_age"), "{error}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_invalid_route_cors_falls_back_to_global_config() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-fallback-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n@CORS({ origins: [\"not-a-url\"], methods: [\"GET\"] })\n$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_options(
        &base_url,
        "/data",
        &[
            ("Origin", "https://client.example.com"),
            ("Access-Control-Request-Method", "GET"),
        ],
    );

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
            .map(|(_, value)| value.as_str()),
        Some("*")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn cli_serve_invalid_cors_exits_non_zero_without_running_message() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-cli-cors-invalid-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"*\"], credentials: true })\n$main() {}\n\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("dolang-cli")
        .arg("--")
        .arg("serve")
        .arg(&project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cli should run");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait().expect("cli status should be readable") {
            Some(_) => break,
            None if Instant::now() >= deadline => {
                child.kill().expect("cli should be killable on timeout");
                let output = child
                    .wait_with_output()
                    .expect("timed out cli output should be readable");
                let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
                let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
                panic!(
                    "cli serve test timed out after 10s; process likely hung\nstdout:\n{stdout}\nstderr:\n{stderr}"
                );
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }

    let output = child
        .wait_with_output()
        .expect("cli output should be readable");

    assert!(!output.status.success(), "{output:?}");

    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    assert!(!stdout.contains("Server running at"), "{stdout}");
    assert!(stderr.contains("DOL-C002"), "{stderr}");
    assert!(stderr.contains("credentials: true"), "{stderr}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_request_context_primitives_are_visible_in_real_requests() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-context-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/users/:id\") show_user(id, limit) -> JSON {\n    $# {\n        \"id\": id,\n        \"limit\": limit,\n        \"auth\": $HDR(\"Authorization\")\n    };\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/users/42?limit=10",
        &[("Authorization", "Bearer demo-token")],
    );

    assert_eq!(response.status, 200);
    assert!(response.body.contains("\"id\":\"42\""), "{}", response.body);
    assert!(response.body.contains("\"limit\":\"10\""), "{}", response.body);
    assert!(
        response.body.contains("\"auth\":\"Bearer demo-token\""),
        "{}",
        response.body
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_multiple_path_params_are_visible_in_real_requests() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-path-multi-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/org/:org/repo/:repo\") show_repo(org, repo) -> JSON {\n    $# {\"org\": org, \"repo\": repo};\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/org/acme/repo/dolang");

    assert_eq!(response.status, 200);
    assert!(response.body.contains("\"org\":\"acme\""), "{}", response.body);
    assert!(response.body.contains("\"repo\":\"dolang\""), "{}", response.body);

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_post_body_is_visible_in_real_requests() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-body-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$POST(\"/users\") create_user(body) -> JSON {\n    $# {\"name\": body[\"name\"]};\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers_and_body(
        "POST",
        &base_url,
        "/users",
        &[("Content-Type", "application/json")],
        Some("{\"name\":\"Alice\"}"),
    );

    assert_eq!(response.status, 200);
    assert!(response.body.contains("\"name\":\"Alice\""), "{}", response.body);

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_res_constructor_returns_non_200_status() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-res-status-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/missing\") missing() -> JSON {\n    $# $RES(404, {\"msg\": \"not found\"});\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/missing");

    assert_eq!(response.status, 404);
    assert_eq!(response.body, "{\"msg\":\"not found\"}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_res_constructor_returns_created_status() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-res-created-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$POST(\"/users\") create() -> JSON {\n    $# $RES(201, {\"id\": 1});\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request("POST", &base_url, "/users");

    assert_eq!(response.status, 201);
    assert_eq!(response.body, "{\"id\":1}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_uncaught_throw_returns_http_500() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-throw-500-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/boom\") boom() -> JSON {\n    $throw \"boom\";\n}\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/boom");

    assert_eq!(response.status, 500);
    assert_eq!(response.body, "{\"error\":\"uncaught throw: boom\"}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_credentials_cors_emits_allow_credentials() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-credentials-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"https://client.example.com\"], methods: [\"GET\"], credentials: true })\n$main() {}\n\n$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://client.example.com")],
    );

    assert_eq!(response.status, 200);
    assert_eq!(
        response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Credentials"))
            .map(|(_, value)| value.as_str()),
        Some("true")
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_without_cors_does_not_emit_cors_headers() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-none-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$GET(\"/data\") data() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/data",
        &[("Origin", "https://client.example.com")],
    );

    assert_eq!(response.status, 200);
    assert!(
        response
            .headers
            .iter()
            .all(|(name, _)| !name.to_ascii_lowercase().starts_with("access-control-")),
        "{:?}",
        response.headers
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_cors_and_set_hdr_can_be_combined_in_any_order() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-cors-hdr-combo-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS({ origins: [\"https://client.example.com\"], methods: [\"GET\"] })\n@SET_HDR({ \"X-App\": \"combo-a\" })\n$GET(\"/a\") a() -> String { $# \"ok\"; }\n\n@SET_HDR({ \"X-App\": \"combo-b\" })\n@CORS({ origins: [\"https://client.example.com\"], methods: [\"GET\"] })\n$GET(\"/b\") b() -> String { $# \"ok\"; }\n",
    )
    .expect("main");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    for (path, expected_header) in [("/a", "combo-a"), ("/b", "combo-b")] {
        let response = http_request_with_headers(
            "GET",
            &base_url,
            path,
            &[("Origin", "https://client.example.com")],
        );

        assert_eq!(response.status, 200, "path {path}");
        assert_eq!(
            response
                .headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("Access-Control-Allow-Origin"))
                .map(|(_, value)| value.as_str()),
            Some("https://client.example.com"),
            "path {path}"
        );
        assert_eq!(
            response
                .headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("X-App"))
                .map(|(_, value)| value.as_str()),
            Some(expected_header),
            "path {path}"
        );
    }

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_linked_module_routes_keep_prefixes() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-live-http-link-{unique}"));
    fs::create_dir_all(project_dir.join("routers")).expect("routers dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"routers.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("routers/api.dol"),
        "$GET(\"/health\") health() -> String { $# \"ok\"; }\n",
    )
    .expect("router");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/v1/health");

    assert_eq!(response.status, 200);
    assert_eq!(response.body, "{\"value\":\"ok\"}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
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
