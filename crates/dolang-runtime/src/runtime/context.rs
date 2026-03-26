use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::interpreter::{HttpRoute, StaticRoute};

use std::collections::HashMap;

/// A single field in a registered type shape.
#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub type_name: String,
    pub optional: bool,
}

/// Shape descriptor registered by `$Type` declarations.
#[derive(Debug, Clone)]
pub struct TypeShape {
    pub name: String,
    pub fields: Vec<TypeField>,
}

use super::intrinsics::{
    IntrinsicRegistry, NativeFn, NativeFnMap, NativeModuleRegistry, RuntimePolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    Repl,
    Run,
    Serve,
    Test,
}

#[derive(Clone)]
pub struct RuntimeContext {
    mode: RuntimeMode,
    project_root: PathBuf,
    current_file: Option<String>,
    project_config: Option<ProjectConfig>,
    intrinsic_registry: IntrinsicRegistry,
    runtime_policy: RuntimePolicy,
    http_routes: Vec<HttpRoute>,
    static_routes: Vec<StaticRoute>,
    /// Callable native functions accessible directly by name from Dolang code.
    native_fn_registry: HashMap<String, NativeFn>,
    /// Native modules importable via `$mod path;`.
    native_module_registry: NativeModuleRegistry,
    /// Type shapes registered by `$Type` declarations.
    type_registry: HashMap<String, TypeShape>,
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
            native_fn_registry: HashMap::new(),
            native_module_registry: NativeModuleRegistry::new(),
            type_registry: HashMap::new(),
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
        id: &str,
        args: &[crate::interpreter::DolangValue],
    ) -> Result<crate::interpreter::DolangValue, crate::error::Error> {
        self.runtime_policy.check(id)?;
        self.intrinsic_registry.call(id, args, self)
    }

    /// Register a native (Rust) function callable from Dolang by name.
    pub fn register_native_fn(&mut self, name: impl Into<String>, f: NativeFn) {
        self.native_fn_registry.insert(name.into(), f);
    }

    /// Look up a registered native function by name.
    pub fn get_native_fn(&self, name: &str) -> Option<&NativeFn> {
        self.native_fn_registry.get(name)
    }

    /// Register a native module importable via `$mod path;`.
    pub fn register_native_module(&mut self, path: &str, exports: NativeFnMap) {
        self.native_module_registry.register(path, exports);
    }

    /// Look up a registered native module by path.
    pub fn native_module(&self, path: &str) -> Option<&NativeFnMap> {
        self.native_module_registry.get(path)
    }

    /// Register a type shape from a `$Type` declaration.
    pub fn register_type(&mut self, name: impl Into<String>, shape: TypeShape) {
        self.type_registry.insert(name.into(), shape);
    }

    /// Look up a registered type shape by name.
    pub fn get_type(&self, name: &str) -> Option<&TypeShape> {
        self.type_registry.get(name)
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

impl std::fmt::Debug for RuntimeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeContext")
            .field("mode", &self.mode)
            .field("project_root", &self.project_root)
            .field("current_file", &self.current_file)
            .field("native_fns", &self.native_fn_registry.len())
            .finish_non_exhaustive()
    }
}
