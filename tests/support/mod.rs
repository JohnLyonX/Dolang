#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use dolang::interpreter::{DolangValue, HttpRoute};
use dolang::runtime::{
    HandlerInput, ProgramState, RuntimeContext, RuntimeMode, execute_http_route,
    execute_program_with_writer, load_context_and_program,
};

pub struct ScriptOutcome {
    pub state: ProgramState,
    pub context: RuntimeContext,
    pub stdout: String,
    pub error: Option<String>,
}

pub struct RouteOutcome {
    pub route_count: usize,
    pub should_continue: bool,
    pub result: Option<DolangValue>,
    pub error: Option<String>,
}

pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(relative)
}

pub fn read_text(relative: &str) -> String {
    fs::read_to_string(fixture_path(relative)).expect("fixture should exist")
}

pub fn run_fixture(relative: &str, mode: RuntimeMode) -> ScriptOutcome {
    run_program_at_path(&fixture_path(relative), mode)
}

pub fn run_program_at_path(path: &Path, mode: RuntimeMode) -> ScriptOutcome {
    let (mut context, program) = match load_context_and_program(mode.clone(), path) {
        Ok(loaded) => loaded,
        Err(err) => {
            let project_root = if path.is_dir() {
                path.to_path_buf()
            } else {
                path.parent().unwrap_or(Path::new(".")).to_path_buf()
            };
            return ScriptOutcome {
                state: ProgramState::new(),
                context: RuntimeContext::new(mode, project_root),
                stdout: String::new(),
                error: Some(err.to_string()),
            };
        }
    };
    let mut state = ProgramState::new();
    let mut output = Vec::new();

    let error =
        execute_program_with_writer(&program.statements, &mut state, &mut context, &mut output)
            .err()
            .map(|err| err.to_string());

    ScriptOutcome {
        state,
        context,
        stdout: String::from_utf8(output).expect("stdout should be utf-8"),
        error,
    }
}

pub fn run_route_fixture(
    relative: &str,
    method: &str,
    path: &str,
    body: Option<DolangValue>,
) -> RouteOutcome {
    let outcome = run_fixture(relative, RuntimeMode::Test);
    assert!(
        outcome.error.is_none(),
        "route fixture bootstrap should succeed"
    );

    let routes = outcome.context.routes().to_vec();
    let route = routes
        .iter()
        .find(|route| route.method == method && route.path == path)
        .unwrap_or_else(|| panic!("route {method} {path} should exist"));

    let mut input = HandlerInput::new(path);
    input.body = body;

    let (should_continue, result, error) = execute_http_route(route, &input, &outcome.context);

    RouteOutcome {
        route_count: routes.len(),
        should_continue,
        result,
        error,
    }
}

pub fn route_by_signature<'a>(
    routes: &'a [HttpRoute],
    method: &str,
    path: &str,
) -> Option<&'a HttpRoute> {
    routes
        .iter()
        .find(|route| route.method == method && route.path == path)
}
