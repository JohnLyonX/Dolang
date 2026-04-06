use crate::error::Error;
use crate::interpreter::DolangValue;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use super::{RuntimeContext, sql_intrinsics};

/// String-keyed intrinsic identifier — open for extension without modifying core.
pub type IntrinsicId = &'static str;

/// Well-known intrinsic IDs shipped with the runtime.
pub mod ids {
    pub const FS_READ_TEXT: &str = "fs.read_text";
    pub const FS_READ_LINES: &str = "fs.read_lines";
    pub const FS_WRITE_TEXT: &str = "fs.write_text";
    pub const FS_APPEND_TEXT: &str = "fs.append_text";
    pub const FS_DELETE: &str = "fs.delete";
    pub const FS_EXISTS: &str = "fs.exists";
    pub const FS_SIZE: &str = "fs.size";
    pub const FS_IS_DIR: &str = "fs.is_dir";
    pub const FS_LIST: &str = "fs.list";
    pub const FS_MKDIR: &str = "fs.mkdir";
    pub const FS_MKDIR_ALL: &str = "fs.mkdir_all";
    pub const FS_RMDIR: &str = "fs.rmdir";
    pub const FS_COPY: &str = "fs.copy";
    pub const FS_RENAME: &str = "fs.rename";
    pub const ENV_GET: &str = "env.get";
    pub const CONFIG_GET: &str = "config.get";
    pub const SQL_SQLITE_CONNECT: &str = "sql.sqlite.connect";
    pub const SQL_POSTGRES_CONNECT: &str = "sql.postgres.connect";
    pub const SQL_QUERY: &str = "sql.query";
    pub const SQL_EXECUTE: &str = "sql.execute";
    pub const SQL_CLOSE: &str = "sql.close";
}

pub type IntrinsicCall = fn(&[DolangValue], &RuntimeContext) -> Result<DolangValue, Error>;

/// A native Rust function callable from Dolang code (Arc for cheap clone + closure support).
pub type NativeFn =
    Arc<dyn Fn(&[DolangValue], &RuntimeContext) -> Result<DolangValue, Error> + Send + Sync>;

/// Map of function name → native function, used for module exports.
pub type NativeFnMap = HashMap<String, NativeFn>;

#[derive(Debug, Clone, Default)]
pub struct IntrinsicRegistry {
    handlers: HashMap<String, IntrinsicCall>,
}

impl IntrinsicRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    pub fn register(&mut self, id: &str, call: IntrinsicCall) {
        self.handlers.insert(id.to_string(), call);
    }

    pub fn unregister(&mut self, id: &str) {
        self.handlers.remove(id);
    }

    pub fn call(
        &self,
        id: &str,
        args: &[DolangValue],
        context: &RuntimeContext,
    ) -> Result<DolangValue, Error> {
        let Some(handler) = self.handlers.get(id) else {
            return Err(Error::Interpreter(format!(
                "runtime intrinsic '{}' is not registered",
                id
            )));
        };
        handler(args, context)
    }

    fn register_defaults(&mut self) {
        self.register(ids::FS_READ_TEXT, fs_read_text);
        self.register(ids::FS_READ_LINES, fs_read_lines);
        self.register(ids::FS_WRITE_TEXT, fs_write_text);
        self.register(ids::FS_APPEND_TEXT, fs_append_text);
        self.register(ids::FS_DELETE, fs_delete);
        self.register(ids::FS_EXISTS, fs_exists);
        self.register(ids::FS_SIZE, fs_size);
        self.register(ids::FS_IS_DIR, fs_is_dir);
        self.register(ids::FS_LIST, fs_list);
        self.register(ids::FS_MKDIR, fs_mkdir);
        self.register(ids::FS_MKDIR_ALL, fs_mkdir_all);
        self.register(ids::FS_RMDIR, fs_rmdir);
        self.register(ids::FS_COPY, fs_copy);
        self.register(ids::FS_RENAME, fs_rename);
        self.register(ids::ENV_GET, env_get);
        self.register(ids::CONFIG_GET, config_get);
        self.register(ids::SQL_SQLITE_CONNECT, sql_intrinsics::sqlite_connect);
        self.register(ids::SQL_POSTGRES_CONNECT, sql_intrinsics::postgres_connect);
        self.register(ids::SQL_QUERY, sql_intrinsics::sql_query);
        self.register(ids::SQL_EXECUTE, sql_intrinsics::sql_execute);
        self.register(ids::SQL_CLOSE, sql_intrinsics::sql_close);
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimePolicy {
    denied: HashSet<String>,
}

impl RuntimePolicy {
    pub fn allow_all() -> Self {
        Self::default()
    }

    pub fn deny(&mut self, id: &str) {
        self.denied.insert(id.to_string());
    }

    pub fn allows(&self, id: &str) -> bool {
        !self.denied.contains(id)
    }

    pub fn check(&self, id: &str) -> Result<(), Error> {
        if self.allows(id) {
            Ok(())
        } else {
            Err(Error::Interpreter(format!(
                "runtime intrinsic '{}' is denied by runtime policy",
                id
            )))
        }
    }
}

pub fn intrinsic_string_arg<'a>(
    id: &str,
    args: &'a [DolangValue],
    index: usize,
) -> Result<&'a str, Error> {
    let Some(value) = args.get(index) else {
        return Err(Error::Interpreter(format!(
            "runtime intrinsic '{}' expects at least {} argument(s), got {}",
            id,
            index + 1,
            args.len()
        )));
    };

    match value {
        DolangValue::Str(s) => Ok(s.as_str()),
        other => Err(Error::Interpreter(format!(
            "runtime intrinsic '{}' expects String at argument {}, got {}",
            id,
            index,
            other.type_name()
        ))),
    }
}

pub fn intrinsic_path_buf_arg(
    id: &str,
    args: &[DolangValue],
    index: usize,
) -> Result<PathBuf, Error> {
    Ok(PathBuf::from(intrinsic_string_arg(id, args, index)?))
}

fn resolve_fs_path(context: &RuntimeContext, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        context.project_root().join(path)
    }
}

fn fs_read_text(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_READ_TEXT, args, 0)?);
    match fs::read_to_string(&path) {
        Ok(content) => Ok(DolangValue::Str(content)),
        Err(err) => Err(Error::Interpreter(format!("cannot read file: {}", err))),
    }
}

fn fs_read_lines(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(
        context,
        intrinsic_path_buf_arg(ids::FS_READ_LINES, args, 0)?,
    );
    match fs::read_to_string(&path) {
        Ok(content) => Ok(DolangValue::List(
            content
                .lines()
                .map(|line| DolangValue::Str(line.to_string()))
                .collect(),
        )),
        Err(err) => Err(Error::Interpreter(format!("cannot read file: {}", err))),
    }
}

fn fs_write_text(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(
        context,
        intrinsic_path_buf_arg(ids::FS_WRITE_TEXT, args, 0)?,
    );
    let content = intrinsic_string_arg(ids::FS_WRITE_TEXT, args, 1)?;
    match fs::write(&path, content) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                Err(Error::Interpreter(format!(
                    "directory not found for path: {}",
                    path.display()
                )))
            } else {
                Err(Error::Interpreter(format!("write error: {}", err)))
            }
        }
    }
}

fn fs_append_text(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(
        context,
        intrinsic_path_buf_arg(ids::FS_APPEND_TEXT, args, 0)?,
    );
    let content = intrinsic_string_arg(ids::FS_APPEND_TEXT, args, 1)?;

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(DolangValue::Null),
            Err(err) => Err(Error::Interpreter(format!("write error: {}", err))),
        },
        Err(err) => Err(Error::Interpreter(format!("cannot open file: {}", err))),
    }
}

fn fs_delete(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_DELETE, args, 0)?);
    match fs::remove_file(&path) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!("cannot delete file: {}", err))),
    }
}

fn fs_exists(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_EXISTS, args, 0)?);
    Ok(DolangValue::Bool(fs::metadata(path).is_ok()))
}

fn fs_size(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_SIZE, args, 0)?);
    match fs::metadata(path) {
        Ok(metadata) => Ok(DolangValue::Int(metadata.len() as i64)),
        Err(_) => Ok(DolangValue::Int(0)),
    }
}

fn fs_is_dir(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_IS_DIR, args, 0)?);
    match fs::metadata(path) {
        Ok(metadata) => Ok(DolangValue::Bool(metadata.is_dir())),
        Err(_) => Ok(DolangValue::Bool(false)),
    }
}

fn fs_list(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_LIST, args, 0)?);
    match fs::read_dir(&path) {
        Ok(entries) => {
            let mut names = Vec::new();
            for entry in entries {
                match entry {
                    Ok(e) => names.push(DolangValue::Str(
                        e.file_name().to_string_lossy().to_string(),
                    )),
                    Err(err) => {
                        return Err(Error::Interpreter(format!("cannot read entry: {}", err)));
                    }
                }
            }
            Ok(DolangValue::List(names))
        }
        Err(err) => Err(Error::Interpreter(format!(
            "cannot list directory: {}",
            err
        ))),
    }
}

fn fs_mkdir(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_MKDIR, args, 0)?);
    match fs::create_dir(&path) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!(
            "cannot create directory: {}",
            err
        ))),
    }
}

fn fs_mkdir_all(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_MKDIR_ALL, args, 0)?);
    match fs::create_dir_all(&path) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!(
            "cannot create directories: {}",
            err
        ))),
    }
}

fn fs_rmdir(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_RMDIR, args, 0)?);
    match fs::remove_dir(&path) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!(
            "cannot remove directory: {}",
            err
        ))),
    }
}

fn fs_copy(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let src = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_COPY, args, 0)?);
    let dst = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_COPY, args, 1)?);
    match fs::copy(&src, &dst) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!("cannot copy file: {}", err))),
    }
}

fn fs_rename(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let src = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_RENAME, args, 0)?);
    let dst = resolve_fs_path(context, intrinsic_path_buf_arg(ids::FS_RENAME, args, 1)?);
    match fs::rename(&src, &dst) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!("cannot rename: {}", err))),
    }
}

fn env_get(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let key = intrinsic_string_arg(ids::ENV_GET, args, 0)?;
    match std::env::var(key) {
        Ok(value) => Ok(DolangValue::Str(value)),
        Err(_) => Err(Error::Interpreter(format!(
            "environment variable '{}' is not defined",
            key
        ))),
    }
}

fn config_get(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let key = intrinsic_string_arg(ids::CONFIG_GET, args, 0)?;

    if !context.is_serve_mode() {
        return Err(Error::Interpreter(
            "$<<CONFIG() is only available in serve mode".to_string(),
        ));
    }

    let Some(config) = context.project_config() else {
        return Err(Error::Interpreter("config not loaded".to_string()));
    };

    match config.get(key) {
        Some(value) => Ok(DolangValue::Str(value.to_string())),
        None => Err(Error::Interpreter(format!("config '{}' not found", key))),
    }
}

/// Registry of native modules that can be imported via `$mod path;`.
/// Maps module path (e.g. `"std.fs"`) to a map of function name → NativeFn.
#[derive(Default, Clone)]
pub struct NativeModuleRegistry {
    modules: HashMap<String, NativeFnMap>,
}

impl std::fmt::Debug for NativeModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeModuleRegistry({} modules)", self.modules.len())
    }
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, path: &str, exports: NativeFnMap) {
        self.modules.insert(path.to_string(), exports);
    }

    pub fn get(&self, path: &str) -> Option<&NativeFnMap> {
        self.modules.get(path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::module::ProjectConfig;

    use super::*;
    use super::{IntrinsicRegistry, RuntimePolicy, ids};
    use crate::runtime::{RuntimeContext, RuntimeMode};

    fn temp_path(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("dolang-intrinsic-{prefix}-{unique}.txt"))
    }

    fn test_context(mode: RuntimeMode) -> RuntimeContext {
        RuntimeContext::new(mode, PathBuf::from("."))
    }

    #[test]
    fn registry_calls_registered_intrinsic() {
        fn echo(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
            Ok(args.first().cloned().unwrap_or(DolangValue::Null))
        }

        let mut registry = IntrinsicRegistry::new();
        registry.register(ids::FS_READ_TEXT, echo);

        let context = test_context(RuntimeMode::Test);
        let value = registry
            .call(
                ids::FS_READ_TEXT,
                &[DolangValue::Str("ok".to_string())],
                &context,
            )
            .expect("intrinsic should run");

        assert_eq!(value, DolangValue::Str("ok".to_string()));
    }

    #[test]
    fn unregistered_intrinsic_returns_error() {
        let registry = IntrinsicRegistry::new();
        let context = test_context(RuntimeMode::Test);
        let error = registry
            .call(ids::FS_READ_TEXT, &[], &context)
            .expect_err("missing intrinsic should fail");
        assert!(error.to_string().contains("is not registered"));
    }

    #[test]
    fn runtime_policy_can_deny_intrinsic() {
        let mut policy = RuntimePolicy::allow_all();
        policy.deny(ids::FS_READ_TEXT);

        let error = policy
            .check(ids::FS_READ_TEXT)
            .expect_err("denied intrinsic should fail");
        assert!(error.to_string().contains("denied by runtime policy"));
    }

    #[test]
    fn intrinsic_argument_validation_reports_type_errors() {
        let error = intrinsic_string_arg(ids::FS_READ_TEXT, &[DolangValue::Int(1)], 0)
            .expect_err("non-string argument should fail");
        assert!(error.to_string().contains("expects String"));
    }

    #[test]
    fn fs_intrinsics_round_trip() {
        let path = temp_path("roundtrip");
        let context = test_context(RuntimeMode::Test);

        fs_write_text(
            &[
                DolangValue::Str(path.display().to_string()),
                DolangValue::Str("hello\nworld".to_string()),
            ],
            &context,
        )
        .expect("write should succeed");

        let content = fs_read_text(&[DolangValue::Str(path.display().to_string())], &context)
            .expect("read should succeed");
        let lines = fs_read_lines(&[DolangValue::Str(path.display().to_string())], &context)
            .expect("read lines should succeed");
        let exists = fs_exists(&[DolangValue::Str(path.display().to_string())], &context)
            .expect("exists should succeed");

        assert_eq!(content, DolangValue::Str("hello\nworld".to_string()));
        assert_eq!(
            lines,
            DolangValue::List(vec![
                DolangValue::Str("hello".to_string()),
                DolangValue::Str("world".to_string())
            ])
        );
        assert_eq!(exists, DolangValue::Bool(true));

        fs_delete(&[DolangValue::Str(path.display().to_string())], &context)
            .expect("delete should succeed");
    }

    #[test]
    fn fs_directory_ops_create_list_remove() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dolang-dir-test-{}", unique));
        let context = test_context(RuntimeMode::Test);

        // mkdir
        fs_mkdir(&[DolangValue::Str(dir.display().to_string())], &context)
            .expect("mkdir should succeed");
        assert!(dir.is_dir());

        // write a file inside, then list
        let file_path = dir.join("hello.txt");
        fs_write_text(
            &[
                DolangValue::Str(file_path.display().to_string()),
                DolangValue::Str("hi".to_string()),
            ],
            &context,
        )
        .expect("write should succeed");

        let entries = fs_list(&[DolangValue::Str(dir.display().to_string())], &context)
            .expect("list should succeed");
        assert_eq!(
            entries,
            DolangValue::List(vec![DolangValue::Str("hello.txt".to_string())])
        );

        // copy file
        let copy_path = dir.join("copy.txt");
        fs_copy(
            &[
                DolangValue::Str(file_path.display().to_string()),
                DolangValue::Str(copy_path.display().to_string()),
            ],
            &context,
        )
        .expect("copy should succeed");
        assert!(copy_path.exists());

        // rename file
        let renamed_path = dir.join("renamed.txt");
        fs_rename(
            &[
                DolangValue::Str(copy_path.display().to_string()),
                DolangValue::Str(renamed_path.display().to_string()),
            ],
            &context,
        )
        .expect("rename should succeed");
        assert!(renamed_path.exists());
        assert!(!copy_path.exists());

        // cleanup: delete files then rmdir
        fs_delete(
            &[DolangValue::Str(file_path.display().to_string())],
            &context,
        )
        .expect("delete file should succeed");
        fs_delete(
            &[DolangValue::Str(renamed_path.display().to_string())],
            &context,
        )
        .expect("delete renamed should succeed");
        fs_rmdir(&[DolangValue::Str(dir.display().to_string())], &context)
            .expect("rmdir should succeed");
        assert!(!dir.exists());
    }

    #[test]
    fn fs_mkdir_all_creates_nested_directories() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("dolang-mkdirall-{}", unique));
        let nested = base.join("a").join("b").join("c");
        let context = test_context(RuntimeMode::Test);

        fs_mkdir_all(&[DolangValue::Str(nested.display().to_string())], &context)
            .expect("mkdir_all should succeed");
        assert!(nested.is_dir());

        // cleanup
        std::fs::remove_dir_all(&base).expect("cleanup should succeed");
    }

    #[test]
    fn env_get_intrinsic_uses_process_environment() {
        let context = test_context(RuntimeMode::Test);
        let existing = std::env::var("HOME")
            .ok()
            .map(|_| "HOME")
            .or_else(|| std::env::var("PATH").ok().map(|_| "PATH"))
            .expect("HOME or PATH should be available");

        let value = env_get(&[DolangValue::Str(existing.to_string())], &context)
            .expect("env should be readable");
        assert!(matches!(value, DolangValue::Str(_)));
    }

    #[test]
    fn config_get_requires_serve_mode_and_loaded_config() {
        let non_serve_context = test_context(RuntimeMode::Test);
        let error = config_get(
            &[DolangValue::Str("APP_ENV".to_string())],
            &non_serve_context,
        )
        .expect_err("config should be disabled outside serve mode");
        assert!(error.to_string().contains("only available in serve mode"));

        let mut serve_context = test_context(RuntimeMode::Serve);
        serve_context.set_project_config(Some(
            ProjectConfig::parse_toml(
                "name = \"demo\"\nversion = \"0.1.0\"\n[env]\nAPP_ENV = \"dev\"\n",
            )
            .expect("config should parse"),
        ));

        let value = config_get(&[DolangValue::Str("APP_ENV".to_string())], &serve_context)
            .expect("config should load in serve mode");
        assert_eq!(value, DolangValue::Str("dev".to_string()));
    }
}
