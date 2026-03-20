use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::interpreter::{HttpRoute, StaticRoute};

use super::intrinsics::{IntrinsicId, IntrinsicRegistry, RuntimePolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    Repl,
    Run,
    Serve,
    Test,
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    mode: RuntimeMode,
    project_root: PathBuf,
    current_file: Option<String>,
    project_config: Option<ProjectConfig>,
    intrinsic_registry: IntrinsicRegistry,
    runtime_policy: RuntimePolicy,
    http_routes: Vec<HttpRoute>,
    static_routes: Vec<StaticRoute>,
}

impl RuntimeContext {
    pub fn new(mode: RuntimeMode, project_root: PathBuf) -> Self {
        Self {
            mode,
            project_root,
            current_file: None,
            project_config: None,
            intrinsic_registry: IntrinsicRegistry::with_defaults(),
            runtime_policy: RuntimePolicy::allow_all(),
            http_routes: Vec::new(),
            static_routes: Vec::new(),
        }
    }

    pub fn mode(&self) -> &RuntimeMode {
        &self.mode
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn current_file(&self) -> Option<&str> {
        self.current_file.as_deref()
    }

    pub fn set_current_file(&mut self, filename: Option<String>) {
        self.current_file = filename;
    }

    pub fn set_project_config(&mut self, project_config: Option<ProjectConfig>) {
        self.project_config = project_config;
    }

    pub fn is_serve_mode(&self) -> bool {
        matches!(self.mode, RuntimeMode::Serve)
    }

    pub fn project_config(&self) -> Option<&ProjectConfig> {
        self.project_config.as_ref()
    }

    pub fn set_intrinsic_registry(&mut self, intrinsic_registry: IntrinsicRegistry) {
        self.intrinsic_registry = intrinsic_registry;
    }

    pub fn set_runtime_policy(&mut self, runtime_policy: RuntimePolicy) {
        self.runtime_policy = runtime_policy;
    }

    pub fn call_intrinsic(
        &self,
        id: IntrinsicId,
        args: &[crate::interpreter::DolangValue],
    ) -> Result<crate::interpreter::DolangValue, crate::error::Error> {
        self.runtime_policy.check(id)?;
        self.intrinsic_registry.call(id, args, self)
    }

    pub fn clear_routes(&mut self) {
        self.http_routes.clear();
        self.static_routes.clear();
    }

    pub fn register_http_route(&mut self, route: HttpRoute) {
        self.http_routes.push(route);
    }

    pub fn extend_http_routes(&mut self, routes: Vec<HttpRoute>) {
        self.http_routes.extend(routes);
    }

    pub fn register_static_route(&mut self, route: StaticRoute) {
        self.static_routes.push(route);
    }

    pub fn routes(&self) -> &[HttpRoute] {
        &self.http_routes
    }

    pub fn static_routes(&self) -> &[StaticRoute] {
        &self.static_routes
    }

    pub fn server_host(&self) -> &str {
        self.project_config
            .as_ref()
            .map(|cfg| cfg.server.host.as_str())
            .unwrap_or("0.0.0.0")
    }

    pub fn server_port(&self) -> u16 {
        self.project_config
            .as_ref()
            .map(|cfg| cfg.server.port)
            .unwrap_or(8080)
    }

    pub fn print_routes(&self) {
        if self.http_routes.is_empty() {
            println!("No routes registered.");
            return;
        }

        println!("Registered Routes:");
        println!("{:<8} {:<30} → Handler", "Method", "Path");
        println!("{:<8} {:<30} → --------", "------", "----");
        for route in &self.http_routes {
            println!("{:<8} {:<30} → {}", route.method, route.path, route.name);
        }
    }
}
