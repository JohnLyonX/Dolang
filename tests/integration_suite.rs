mod integration;
mod support;

use std::fs;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dolang::interpreter::DolangValue;
use dolang::module::ProjectConfig;
use dolang::parser;
use dolang::runtime::RuntimeMode;
use dolang::runtime::auth::{RuntimeAuthConfig, SessionStore, sign_jwt};
use dolang::runtime::{
    HandlerInput, ProgramState, RuntimeContext, execute_http_route_in_context,
    execute_program_with_writer,
};

use integration::{assert_fixture_stdout, assert_route_string};
use support::{
    http_get, http_options, http_request, http_request_with_headers,
    http_request_with_headers_and_body, route_by_signature, run_fixture, run_program_at_path,
    start_live_http_server, start_live_http_server_from_context,
    try_start_live_http_server_from_context,
};

fn write_temp_auth_project(package_toml: &str, main_dol: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-auth-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(project_dir.join("package.toml"), package_toml).expect("package.toml");
    fs::write(project_dir.join("main.dol"), main_dol).expect("main.dol");
    project_dir
}

#[test]
fn runtime_fixtures_cover_core_semantics() {
    assert_fixture_stdout("fixtures/runtime/variables_consts.dol", "3\n10\n");
    assert_fixture_stdout("fixtures/runtime/type_annotations.dol", "Int\nString\n");
    assert_fixture_stdout("fixtures/runtime/functions_control_flow.dol", "pass\n");
    assert_fixture_stdout("fixtures/runtime/collections_builtins.dol", "3\ntrue\n2\n");
}

#[test]
fn auth_stdlib_fixtures_cover_core_flows() {
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_jwt_flow.dol",
        "user_1\nadmin\npost:create\n",
    );
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_jwt_refresh_flow.dol",
        "refresh\nuser_1\n",
    );
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_jwt_pair_flow.dol",
        "access\nrefresh\nuser_1\n",
    );
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_session_flow.dol",
        "user_1\ntrue\ntrue\nadmin\n",
    );
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_session_mutation_flow.dol",
        "t_001\nc_001\nnull\ntrue\n",
    );
    assert_fixture_stdout(
        "spec/valid/stdlib/auth_guard_flow.dol",
        "false\ntrue\ntrue\ntrue\ntrue\nuser_1\n",
    );
}

#[test]
fn auth_guard_require_role_fixture_fails() {
    let outcome = run_fixture(
        "spec/invalid/stdlib/auth_guard_require_role_fails.dol",
        RuntimeMode::Test,
    );

    let error = outcome
        .error
        .expect("auth guard failure fixture should fail");
    assert!(error.contains("auth.guard.require_role: missing required role 'admin'"));
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
fn http_handler_return_type_mismatch_fails_in_test_mode_bootstrap() {
    let outcome = run_fixture(
        "spec/invalid/http/http_handler_return_type_mismatch.dol",
        RuntimeMode::Test,
    );

    let error = outcome
        .error
        .expect("http return type mismatch fixture should fail");
    assert!(error.contains("http handler 'get_user' expects return type 'Int' but got 'Map'"));
}

#[test]
fn http_handler_user_return_type_bootstraps_in_test_mode() {
    let outcome = run_fixture(
        "spec/valid/http/http_handler_user_return_type.dol",
        RuntimeMode::Test,
    );
    assert!(
        outcome.error.is_none(),
        "visible user return type should pass"
    );
}

#[test]
fn http_handler_unknown_user_type_fails_in_test_mode_bootstrap() {
    let outcome = run_fixture(
        "spec/invalid/http/http_handler_unknown_user_type_without_import.dol",
        RuntimeMode::Test,
    );

    let error = outcome
        .error
        .expect("unknown user return type fixture should fail");
    assert!(error.contains("http handler 'get_user' references unknown return type 'User'"));
}

#[test]
fn http_fixture_still_bootstraps_when_probe_inputs_are_sufficient() {
    let outcome = run_fixture("fixtures/http/echo_route.dol", RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http fixture should execute");
    assert_eq!(outcome.context.routes().len(), 2);
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
fn live_http_session_protected_route_returns_401_without_cookie() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-session"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"
ttl_seconds = 86400

[server.auth.session.store]
driver = "memory"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": ["admin"],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);
    let response = http_get(&base_url, "/admin");
    let response_json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be json");

    assert_eq!(response.status, 401);
    assert_eq!(response_json["status"], 401);
    assert_eq!(response_json["code"], "auth_unauthorized");
    assert_eq!(response_json["message"], "authentication required");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_login_sets_cookie_and_allows_protected_route() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-session"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"
ttl_seconds = 86400

[server.auth.session.store]
driver = "memory"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": ["admin"],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let session_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let admin =
        http_request_with_headers("GET", &base_url, "/admin", &[("Cookie", &session_cookie)]);

    assert_eq!(login.status, 200);
    assert_eq!(admin.status, 200);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_bearer_token_allows_protected_route() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-bearer"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
roles_any = ["admin"]
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $ token = jwt.sign({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": ["post:create"]
    });
    $# {"token": token};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    let token_json: serde_json::Value =
        serde_json::from_str(&token_response.body).expect("token body should be json");
    let token = token_json["token"]
        .as_str()
        .expect("token should be string");
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/admin",
        &[("Authorization", &format!("Bearer {token}"))],
    );

    assert_eq!(response.status, 200);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_role_guard_returns_403_for_authenticated_but_unauthorized_user() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-bearer"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
roles_any = ["admin"]
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $ token = jwt.sign({
        "sub": "user_2",
        "roles": ["viewer"],
        "permissions": []
    });
    $# {"token": token};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    let token_json: serde_json::Value =
        serde_json::from_str(&token_response.body).expect("token body should be json");
    let token = token_json["token"]
        .as_str()
        .expect("token should be string");
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/admin",
        &[("Authorization", &format!("Bearer {token}"))],
    );
    let response_json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be json");

    assert_eq!(response.status, 403);
    assert_eq!(response_json["status"], 403);
    assert_eq!(response_json["code"], "auth_forbidden");
    assert_eq!(response_json["message"], "forbidden");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_authorization_default_authenticated_requires_login() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-default"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"
ttl_seconds = 86400

[server.auth.session.store]
driver = "memory"

[server.auth.authorization]
enabled = true
default = "authenticated"

[[server.auth.authorization.rules]]
method = "GET"
path = "/login"
require = "public"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/me") me() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let unauthorized = http_get(&base_url, "/me");
    let login = http_get(&base_url, "/login");
    let unauthorized_json: serde_json::Value =
        serde_json::from_str(&unauthorized.body).expect("response should be json");
    let session_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let authorized =
        http_request_with_headers("GET", &base_url, "/me", &[("Cookie", &session_cookie)]);

    assert_eq!(unauthorized.status, 401);
    assert_eq!(unauthorized_json["status"], 401);
    assert_eq!(unauthorized_json["code"], "auth_unauthorized");
    assert_eq!(unauthorized_json["message"], "authentication required");
    assert_eq!(authorized.status, 200);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_permission_guard_returns_403_for_missing_required_permission() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-permissions"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/publish"
require = "authenticated"
permissions_all = ["post:create", "post:publish"]
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $ token = jwt.sign({
        "sub": "user_1",
        "roles": ["editor"],
        "permissions": ["post:create"]
    });
    $# {"token": token};
}

$GET("/publish") publish() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    let token_json: serde_json::Value =
        serde_json::from_str(&token_response.body).expect("token body should be json");
    let token = token_json["token"]
        .as_str()
        .expect("token should be string");
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/publish",
        &[("Authorization", &format!("Bearer {token}"))],
    );
    let response_json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be json");

    assert_eq!(response.status, 403);
    assert_eq!(response_json["status"], 403);
    assert_eq!(response_json["code"], "auth_forbidden");
    assert_eq!(response_json["message"], "forbidden");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_login_and_logout_emit_configured_cookie_headers() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-cookie"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "demo_session"
cookie_secure = true
cookie_http_only = true
cookie_same_site = "strict"
cookie_path = "/"
ttl_seconds = 900

[server.auth.session.store]
driver = "memory"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/logout") logout() -> JSON {
    session.destroy();
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let login_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let logout =
        http_request_with_headers("GET", &base_url, "/logout", &[("Cookie", &login_cookie)]);
    let logout_cookie = logout
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("logout should clear cookie");

    assert!(login_cookie.contains("demo_session=sess_"));
    assert!(login_cookie.contains("Path=/"));
    assert!(login_cookie.contains("Max-Age=900"));
    assert!(login_cookie.contains("SameSite=Strict"));
    assert!(login_cookie.contains("HttpOnly"));
    assert!(login_cookie.contains("Secure"));

    assert!(logout_cookie.contains("demo_session="));
    assert!(logout_cookie.contains("Max-Age=0"));
    assert!(logout_cookie.contains("SameSite=Strict"));
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_bearer_helpers_are_visible_in_request_context() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-bearer-context"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $ token = jwt.sign({
        "sub": "user_9",
        "roles": ["reader"],
        "permissions": ["post:read"]
    });
    $# {"token": token};
}

$GET("/whoami") whoami() -> JSON {
    $# {
        "sub": jwt.current()["sub"],
        "bearer": jwt.bearer()
    };
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    let token_json: serde_json::Value =
        serde_json::from_str(&token_response.body).expect("token body should be json");
    let token = token_json["token"]
        .as_str()
        .expect("token should be string");
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/whoami",
        &[("Authorization", &format!("Bearer {token}"))],
    );
    let response_json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be json");

    assert_eq!(response.status, 200);
    assert_eq!(response_json["sub"], "user_9");
    assert_eq!(response_json["bearer"], token);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_refresh_pair_issues_new_access_token_for_protected_route() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-refresh-pair"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600
refresh_ttl_seconds = 86400

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
roles_any = ["admin"]
"#,
        r#"
$mod std.auth.jwt;

$GET("/login") login() -> JSON {
    $# jwt.issue_pair({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": ["post:create"]
    });
}

$POST("/refresh") refresh() -> JSON {
    $# jwt.refresh_pair(body["refresh_token"]);
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let login_json: serde_json::Value =
        serde_json::from_str(&login.body).expect("login body should be json");
    let refresh_token = login_json["refresh_token"]
        .as_str()
        .expect("refresh token should be string");

    let refresh_body = serde_json::json!({ "refresh_token": refresh_token }).to_string();
    let refresh = http_request_with_headers_and_body(
        "POST",
        &base_url,
        "/refresh",
        &[("Content-Type", "application/json")],
        Some(refresh_body.as_str()),
    );
    let refresh_json: serde_json::Value =
        serde_json::from_str(&refresh.body).expect("refresh body should be json");
    let access_token = refresh_json["access_token"]
        .as_str()
        .expect("access token should be string");

    let admin = http_request_with_headers(
        "GET",
        &base_url,
        "/admin",
        &[("Authorization", &format!("Bearer {access_token}"))],
    );

    assert_eq!(login.status, 200);
    assert_eq!(refresh.status, 200);
    assert_eq!(admin.status, 200);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_refresh_pair_rejects_reuse_of_old_refresh_token() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-refresh-rotation"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600
refresh_ttl_seconds = 86400
"#,
        r#"
$mod std.auth.jwt;

$GET("/login") login() -> JSON {
    $# jwt.issue_pair({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": ["post:create"]
    });
}

$POST("/refresh") refresh() -> JSON {
    $# jwt.refresh_pair(body["refresh_token"]);
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let login_json: serde_json::Value =
        serde_json::from_str(&login.body).expect("login body should be json");
    let refresh_token = login_json["refresh_token"]
        .as_str()
        .expect("refresh token should be string");

    let refresh_body = serde_json::json!({ "refresh_token": refresh_token }).to_string();
    let first_refresh = http_request_with_headers_and_body(
        "POST",
        &base_url,
        "/refresh",
        &[("Content-Type", "application/json")],
        Some(refresh_body.as_str()),
    );
    let second_refresh = http_request_with_headers_and_body(
        "POST",
        &base_url,
        "/refresh",
        &[("Content-Type", "application/json")],
        Some(refresh_body.as_str()),
    );
    let second_refresh_json: serde_json::Value =
        serde_json::from_str(&second_refresh.body).expect("response should be json");

    assert_eq!(first_refresh.status, 200);
    assert_eq!(second_refresh.status, 401);
    assert_eq!(second_refresh_json["status"], 401);
    assert_eq!(second_refresh_json["code"], "auth_unauthorized");
    assert_eq!(second_refresh_json["message"], "invalid refresh token");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_sqlite_refresh_tokens_persist_and_can_be_revoked() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-auth-refresh-sqlite-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    let sqlite_path = project_dir.join("auth.sqlite3");
    fs::write(
        project_dir.join("package.toml"),
        format!(
            r#"
name = "auth-refresh-sqlite"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.session]
enabled = true

[server.auth.session.store]
driver = "sqlite"

[server.auth.session.store.sqlite]
path = "{}"
table = "auth_sessions"

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600
refresh_ttl_seconds = 86400
"#,
            sqlite_path.display()
        ),
    )
    .expect("package.toml");
    fs::write(
        project_dir.join("main.dol"),
        r#"
$mod std.auth.jwt;

$GET("/login") login() -> JSON {
    $# jwt.issue_pair({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": []
    });
}

$POST("/refresh") refresh() -> JSON {
    $# jwt.refresh_pair(body["refresh_token"]);
}

$POST("/revoke") revoke() -> JSON {
    jwt.revoke_refresh(body["refresh_token"]);
    $# {"ok": true};
}
"#,
    )
    .expect("main.dol");

    let first_outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        first_outcome.error.is_none(),
        "serve bootstrap should succeed"
    );
    let first_base_url = start_live_http_server_from_context(first_outcome.context);

    let login = http_get(&first_base_url, "/login");
    let login_json: serde_json::Value =
        serde_json::from_str(&login.body).expect("login body should be json");
    let first_refresh_token = login_json["refresh_token"]
        .as_str()
        .expect("refresh token should be string")
        .to_string();

    let second_outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        second_outcome.error.is_none(),
        "serve restart should succeed"
    );
    let second_base_url = start_live_http_server_from_context(second_outcome.context);

    let first_refresh_body =
        serde_json::json!({ "refresh_token": first_refresh_token }).to_string();
    let refreshed = http_request_with_headers_and_body(
        "POST",
        &second_base_url,
        "/refresh",
        &[("Content-Type", "application/json")],
        Some(first_refresh_body.as_str()),
    );
    let refreshed_json: serde_json::Value =
        serde_json::from_str(&refreshed.body).expect("refresh body should be json");
    let second_refresh_token = refreshed_json["refresh_token"]
        .as_str()
        .expect("refresh token should be string")
        .to_string();

    let third_outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        third_outcome.error.is_none(),
        "serve restart after refresh should succeed"
    );
    let third_base_url = start_live_http_server_from_context(third_outcome.context);

    let revoke_body = serde_json::json!({ "refresh_token": second_refresh_token }).to_string();
    let revoke = http_request_with_headers_and_body(
        "POST",
        &third_base_url,
        "/revoke",
        &[("Content-Type", "application/json")],
        Some(revoke_body.as_str()),
    );

    let fourth_outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        fourth_outcome.error.is_none(),
        "serve restart after revoke should succeed"
    );
    let fourth_base_url = start_live_http_server_from_context(fourth_outcome.context);

    let revoked = http_request_with_headers_and_body(
        "POST",
        &fourth_base_url,
        "/refresh",
        &[("Content-Type", "application/json")],
        Some(revoke_body.as_str()),
    );
    let revoked_json: serde_json::Value =
        serde_json::from_str(&revoked.body).expect("response should be json");

    assert_eq!(refreshed.status, 200);
    assert_eq!(revoke.status, 200);
    assert_eq!(revoked.status, 401);
    assert_eq!(revoked_json["status"], 401);
    assert_eq!(revoked_json["code"], "auth_unauthorized");
    assert_eq!(revoked_json["message"], "invalid refresh token");
    assert!(sqlite_path.exists(), "sqlite auth db should be created");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_bearer_auth_respects_hs512_algorithm_config() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-bearer-hs512"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS512"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $# {"token": jwt.sign({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": []
    })};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    let token_json: serde_json::Value =
        serde_json::from_str(&token_response.body).expect("token body should be json");
    let token = token_json["token"]
        .as_str()
        .expect("token should be string");
    let response = http_request_with_headers(
        "GET",
        &base_url,
        "/admin",
        &[("Authorization", &format!("Bearer {token}"))],
    );

    assert_eq!(response.status, 200);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn load_rejects_unsupported_jwt_algorithm() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-invalid-jwt-alg"
version = "0.1.0"
entry = "main.dol"

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.session]
enabled = true

[server.auth.jwt]
enabled = true
algorithm = "RS256"
secret = "dolang-dev-secret"
"#,
        "$main() {}\n",
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);

    assert!(
        outcome
            .error
            .as_deref()
            .is_some_and(|error| error.contains("unsupported JWT algorithm"))
    );
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_more_specific_authorization_rule_overrides_broader_prefix_rule() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-rule-priority"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"

[server.auth.session.store]
driver = "memory"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin/*"
require = "authenticated"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin/health"
require = "public"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/admin/health") health() -> JSON {
    $# {"ok": true};
}

$GET("/admin/users") users() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let health = http_get(&base_url, "/admin/health");
    let users = http_get(&base_url, "/admin/users");

    assert_eq!(health.status, 200);
    assert_eq!(users.status, 401);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_claims_based_authorization_requires_matching_claim_value() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-claims-guard"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = true
issuer = "dolang"
audience = "dolang"
algorithm = "HS256"
secret = "dolang-dev-secret"
access_ttl_seconds = 3600

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/team"
require = "authenticated"
claims_all = { team_id = "t_001" }
"#,
        r#"
$mod std.auth.jwt;

$GET("/token-ok") token_ok() -> JSON {
    $# {"token": jwt.sign({
        "sub": "user_1",
        "roles": [],
        "permissions": [],
        "team_id": "t_001"
    })};
}

$GET("/token-bad") token_bad() -> JSON {
    $# {"token": jwt.sign({
        "sub": "user_2",
        "roles": [],
        "permissions": [],
        "team_id": "t_999"
    })};
}

$GET("/team") team() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let ok_token_response = http_get(&base_url, "/token-ok");
    let ok_token_json: serde_json::Value =
        serde_json::from_str(&ok_token_response.body).expect("token body should be json");
    let ok_token = ok_token_json["token"].as_str().expect("token should be string");

    let bad_token_response = http_get(&base_url, "/token-bad");
    let bad_token_json: serde_json::Value =
        serde_json::from_str(&bad_token_response.body).expect("token body should be json");
    let bad_token = bad_token_json["token"].as_str().expect("token should be string");

    let allowed = http_request_with_headers(
        "GET",
        &base_url,
        "/team",
        &[("Authorization", &format!("Bearer {ok_token}"))],
    );
    let denied = http_request_with_headers(
        "GET",
        &base_url,
        "/team",
        &[("Authorization", &format!("Bearer {bad_token}"))],
    );
    let denied_json: serde_json::Value =
        serde_json::from_str(&denied.body).expect("response should be json");

    assert_eq!(allowed.status, 200);
    assert_eq!(denied.status, 403);
    assert_eq!(denied_json["code"], "auth_forbidden");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn request_session_mutation_is_not_committed_when_handler_errors() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-session-error"
version = "0.1.0"
entry = "main.dol"

[server.auth]
enabled = true

[server.auth.session]
enabled = true
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $throw "boom";
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        outcome.error.is_none(),
        "route fixture bootstrap should succeed"
    );

    let route = route_by_signature(outcome.context.routes(), "GET", "/login")
        .expect("login route should exist")
        .clone();
    let mut request_context = outcome.context.clone_for_request_execution();
    let input = HandlerInput::new("/login");

    let (_should_continue, _result, error) =
        execute_http_route_in_context(&route, &input, &mut request_context);

    assert_eq!(error.as_deref(), Some("uncaught throw: boom"));

    let session_id = request_context
        .with_request_auth_context(|auth| {
            auth.current_session()
                .map(|session| session.session_id.clone())
        })
        .expect("request auth context should be readable")
        .expect("request should still see current session");
    let stored = request_context
        .with_session_store_mut(|store| store.get(&session_id))
        .expect("session store should be readable");

    assert!(stored.is_none(), "failed handler must not commit session");
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_guard_require_role_returns_403_instead_of_500() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-guard-http"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"

[server.auth.session.store]
driver = "memory"
"#,
        r#"
$mod std.auth.session;
$mod std.auth.guard;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": ["viewer"],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/admin") admin() -> JSON {
    guard.require_role("admin");
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let session_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let admin =
        http_request_with_headers("GET", &base_url, "/admin", &[("Cookie", &session_cookie)]);

    assert_eq!(admin.status, 403);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_session_disabled_rejects_session_stdlib_mutation() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-session-disabled"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = false
cookie_name = "dolang_session"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let response = http_get(&base_url, "/login");

    assert_eq!(response.status, 500);
    assert!(
        response
            .headers
            .iter()
            .all(|(name, _)| !name.eq_ignore_ascii_case("set-cookie"))
    );
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_jwt_disabled_rejects_bearer_auth_and_jwt_stdlib() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-jwt-disabled"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = false
issuer = "dolang"
audience = "dolang"
secret = "dolang-dev-secret"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
"#,
        r#"
$mod std.auth.jwt;

$GET("/token") token() -> JSON {
    $# {"token": jwt.sign({
        "sub": "user_1",
        "roles": ["admin"],
        "permissions": []
    })};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let token_response = http_get(&base_url, "/token");
    assert_eq!(token_response.status, 500);

    let token = sign_jwt(
        &RuntimeAuthConfig {
            enabled: true,
            jwt_enabled: true,
            jwt_secret: Some("dolang-dev-secret".to_string()),
            issuer: "dolang".to_string(),
            audience: "dolang".to_string(),
            ..RuntimeAuthConfig::default()
        },
        "user_1",
        &["admin".to_string()],
        &[],
        &Default::default(),
    )
    .expect("token should sign for test setup");
    let admin = http_request_with_headers(
        "GET",
        &base_url,
        "/admin",
        &[("Authorization", &format!("Bearer {token}"))],
    );

    assert_eq!(admin.status, 401);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_session_rotation_always_rotates_cookie_on_authenticated_request() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-session-rotate"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"
rotation = "always"

[server.auth.session.store]
driver = "memory"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/me"
require = "authenticated"
"#,
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": [],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/me") me() -> JSON {
    $# {"id": session.id()};
}
"#,
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let first_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let me = http_request_with_headers("GET", &base_url, "/me", &[("Cookie", &first_cookie)]);
    let rotated_cookie = me
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("authenticated request should rotate cookie");

    assert_eq!(me.status, 200);
    assert_ne!(first_cookie, rotated_cookie);
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn load_rejects_default_bearer_scheme_when_jwt_is_disabled() {
    let project_dir = write_temp_auth_project(
        r#"
name = "auth-invalid-default"
version = "0.1.0"
entry = "main.dol"

[server.auth]
enabled = true
default_scheme = "bearer"
identity_sources = ["bearer"]

[server.auth.jwt]
enabled = false
"#,
        "$main() {}\n",
    );

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);

    assert!(
        outcome
            .error
            .as_deref()
            .is_some_and(|error| error.contains("default scheme"))
    );
    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn live_http_sqlite_session_store_persists_login_flow() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-auth-sqlite-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    let sqlite_path = project_dir.join("auth.sqlite3");
    fs::write(
        project_dir.join("package.toml"),
        format!(
            r#"
name = "auth-sqlite"
version = "0.1.0"
entry = "main.dol"

[server]
host = "127.0.0.1"
port = 8080

[server.auth]
enabled = true
default_scheme = "session"
identity_sources = ["cookie"]

[server.auth.session]
enabled = true
cookie_name = "dolang_session"
ttl_seconds = 86400

[server.auth.session.store]
driver = "sqlite"

[server.auth.session.store.sqlite]
path = "{}"
table = "auth_sessions"

[server.auth.authorization]
enabled = true
default = "public"

[[server.auth.authorization.rules]]
method = "GET"
path = "/admin"
require = "authenticated"
"#,
            sqlite_path.display()
        ),
    )
    .expect("package.toml");
    fs::write(
        project_dir.join("main.dol"),
        r#"
$mod std.auth.session;

$GET("/login") login() -> JSON {
    session.create({
        "subject": "user_1",
        "roles": ["admin"],
        "permissions": []
    });
    $# {"ok": true};
}

$GET("/admin") admin() -> JSON {
    $# {"ok": true};
}
"#,
    )
    .expect("main.dol");

    let outcome = run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(outcome.error.is_none(), "serve bootstrap should succeed");
    let base_url = start_live_http_server_from_context(outcome.context);

    let login = http_get(&base_url, "/login");
    let session_cookie = login
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
        .expect("login should set cookie");
    let admin =
        http_request_with_headers("GET", &base_url, "/admin", &[("Cookie", &session_cookie)]);

    assert_eq!(login.status, 200);
    assert_eq!(admin.status, 200);
    assert!(sqlite_path.exists(), "sqlite session db should be created");
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
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "@SET_HDR({ \"X-Frame-Options\": \"DENY\" })\n$HTTP(\"/api\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
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
fn linked_module_routes_share_module_state_snapshot() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-link-memory-{unique}"));
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/api\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
        "$fn render() -> String { $# \"ok\"; }\n\
$GET(\"/health\") health() -> String { $# render(); }\n\
$GET(\"/ready\") ready() -> String { $# render(); }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");

    let health = route_by_signature(outcome.context.routes(), "GET", "/api/health")
        .expect("health route should exist");
    let ready = route_by_signature(outcome.context.routes(), "GET", "/api/ready")
        .expect("ready route should exist");

    assert!(Arc::ptr_eq(&health.module_state, &ready.module_state));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn linked_module_functions_share_captured_env_snapshot() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-http-fn-memory-{unique}"));
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/api\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
        "$ shared = \"ok\";\n\
$fn render_health() -> String { $# shared; }\n\
$fn render_ready() -> String { $# shared; }\n\
$GET(\"/health\") health() -> String { $# render_health(); }\n\
$GET(\"/ready\") ready() -> String { $# render_ready(); }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");

    let health = route_by_signature(outcome.context.routes(), "GET", "/api/health")
        .expect("health route should exist");
    let render_health = health
        .module_state
        .fns
        .get("render_health")
        .expect("render_health should exist");
    let render_ready = health
        .module_state
        .fns
        .get("render_ready")
        .expect("render_ready should exist");

    assert!(Arc::ptr_eq(
        &render_health.module_env,
        &render_ready.module_env
    ));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn repeated_imports_share_module_namespace_state() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-module-cache-{unique}"));
    fs::create_dir_all(project_dir.join("features")).expect("features dir");
    fs::create_dir_all(project_dir.join("common")).expect("common dir");
    fs::write(
        project_dir.join("main.dol"),
        "$mod features.a;\n$mod features.b;\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("features/a.dol"),
        "$mod common.shared;\n$fn get() -> String { $# shared.name(); }\n",
    )
    .expect("a");
    fs::write(
        project_dir.join("features/b.dol"),
        "$mod common.shared;\n$fn get() -> String { $# shared.name(); }\n",
    )
    .expect("b");
    fs::write(
        project_dir.join("common/shared.dol"),
        "$fn name() -> String { $# \"shared\"; }\n",
    )
    .expect("shared");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "module imports should succeed");

    let a_shared = match outcome.state.env.get("a").expect("a module should exist") {
        DolangValue::ModuleProxy { state, .. } => match state
            .module_env
            .get("shared")
            .expect("a.shared should exist")
        {
            DolangValue::ModuleProxy { state, .. } => Arc::clone(state),
            other => panic!("expected module proxy for a.shared, got {other}"),
        },
        other => panic!("expected module proxy for a, got {other}"),
    };
    let b_shared = match outcome.state.env.get("b").expect("b module should exist") {
        DolangValue::ModuleProxy { state, .. } => match state
            .module_env
            .get("shared")
            .expect("b.shared should exist")
        {
            DolangValue::ModuleProxy { state, .. } => Arc::clone(state),
            other => panic!("expected module proxy for b.shared, got {other}"),
        },
        other => panic!("expected module proxy for b, got {other}"),
    };

    assert!(Arc::ptr_eq(&a_shared, &b_shared));

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn circular_module_imports_fail_fast() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-mod-cycle-{unique}"));
    fs::create_dir_all(project_dir.join("pkg")).expect("pkg dir");
    fs::write(project_dir.join("main.dol"), "$mod pkg.a;\n").expect("main");
    fs::write(project_dir.join("pkg/a.dol"), "$mod pkg.b;\n").expect("a");
    fs::write(project_dir.join("pkg/b.dol"), "$mod pkg.a;\n").expect("b");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    let error = outcome
        .error
        .expect("circular imports should fail instead of recursing");
    assert!(error.contains("circular module import"), "{error}");
    assert!(error.contains("pkg.a"), "{error}");
    assert!(error.contains("pkg.b"), "{error}");

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn function_calls_isolate_mutations_from_captured_env() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-fn-env-isolation-{unique}"));
    fs::create_dir_all(&project_dir).expect("project dir");
    fs::write(
        project_dir.join("main.dol"),
        "$ items = [1, 2];\n\
$fn mutate() -> Int {\n\
    items.push(3);\n\
    $# items.len();\n\
}\n\
$>> mutate();\n\
$>> items.len();\n",
    )
    .expect("main");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "function execution should succeed");
    assert_eq!(outcome.stdout, "3\n2\n");

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
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "@CORS(\"*\")\n$main() {}\n\n@CORS({ origins: [\"https://api.example.com\"], methods: [\"GET\"] })\n$HTTP(\"/api\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
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
    assert!(
        response.body.contains("\"limit\":\"10\""),
        "{}",
        response.body
    );
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
    assert!(
        response.body.contains("\"org\":\"acme\""),
        "{}",
        response.body
    );
    assert!(
        response.body.contains("\"repo\":\"dolang\""),
        "{}",
        response.body
    );

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
    assert!(
        response.body.contains("\"name\":\"Alice\""),
        "{}",
        response.body
    );

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
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
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
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"router.api\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("router/api.dol"),
        "$fn helper() -> Int { $# 1; }\n$GET(\"/health\") health() -> String { $# \"ok\"; }\n$HTTP(\"/admin\") { $GET(\"/stats\") stats() -> String { $# \"stats\"; } }\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Test);
    assert!(outcome.error.is_none(), "http link should succeed");
    assert!(route_by_signature(outcome.context.routes(), "GET", "/v1/health").is_some());
    assert!(route_by_signature(outcome.context.routes(), "GET", "/v1/admin/stats").is_some());

    fs::write(
        project_dir.join("router/empty.dol"),
        "$fn helper() -> Int { $# 1; }\n",
    )
    .expect("empty router");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/v1\").link(\"router.empty\");\n",
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

#[test]
fn serve_mode_postgres_route_can_query_and_close_connection_when_url_is_present() {
    let Some(url) = std::env::var("DOLANG_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-phase5-serve-postgres-{unique}"));
    fs::create_dir_all(&project_dir).expect("temp project dir");

    let manifest = r#"
name = "phase5-serve-postgres"
version = "0.1.0"
entry = "main.dol"
"#;
    fs::write(project_dir.join("package.toml"), manifest).expect("manifest");
    fs::write(
        project_dir.join("main.dol"),
        format!(
            "$mod std.postgres;\n\n$GET(\"/users\") users() -> List<Map> {{\n    $ conn = postgres.connect(\"{url}\");\n    $ rows = conn.query(\"SELECT id, name, email, created_at::text AS created_at FROM users ORDER BY id\", []);\n    $>> conn.close();\n    $# rows;\n}}\n"
        ),
    )
    .expect("main");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        outcome.error.is_none(),
        "serve postgres fixture should boot: {:?}",
        outcome.error
    );

    let base_url = support::start_live_http_server_from_context(outcome.context);
    let response = support::http_get(&base_url, "/users");

    assert_eq!(response.status, 200);
    assert!(
        response.body.contains("\"Alice\""),
        "body={}",
        response.body
    );
    assert!(response.body.contains("\"Bob\""), "body={}", response.body);

    fs::remove_dir_all(&project_dir).expect("cleanup");
}

#[test]
fn serve_mode_linked_route_can_return_imported_user_type() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("dolang-phase5-linked-type-{unique}"));
    fs::create_dir_all(project_dir.join("router")).expect("router dir");
    fs::create_dir_all(project_dir.join("services")).expect("services dir");
    fs::create_dir_all(project_dir.join("data")).expect("data dir");

    let manifest = r#"
name = "phase5-linked-type"
version = "0.1.0"
entry = "main.dol"
"#;
    fs::write(project_dir.join("package.toml"), manifest).expect("manifest");
    fs::write(
        project_dir.join("main.dol"),
        "$HTTP(\"/api\").link(\"router.users\");\n",
    )
    .expect("main");
    fs::write(
        project_dir.join("data/user_types.dol"),
        "$Type User {\n    id: Int\n    name: String\n}\n",
    )
    .expect("types");
    fs::write(
        project_dir.join("services/user_service.dol"),
        "$mod data.user_types;\n\n$fn get_user() -> User {\n    $# User {\n        id: 1,\n        name: \"Alice\",\n    };\n}\n",
    )
    .expect("service");
    fs::write(
        project_dir.join("router/users.dol"),
        "$mod data.user_types;\n$mod services.user_service;\n\n$GET(\"/users\") get_users() -> User {\n    $# user_service.get_user();\n}\n",
    )
    .expect("router");

    let outcome = support::run_program_at_path(&project_dir, RuntimeMode::Serve);
    assert!(
        outcome.error.is_none(),
        "linked typed fixture should boot: {:?}",
        outcome.error
    );

    let base_url = support::start_live_http_server_from_context(outcome.context);
    let response = support::http_get(&base_url, "/api/users");

    assert_eq!(response.status, 200);
    assert!(
        response.body.contains("\"Alice\""),
        "body={}",
        response.body
    );

    fs::remove_dir_all(&project_dir).expect("cleanup");
}
