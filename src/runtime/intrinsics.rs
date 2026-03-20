use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::error::Error;
use crate::interpreter::DolangValue;

use super::RuntimeContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicId {
    FsReadText,
    FsReadLines,
    FsWriteText,
    FsAppendText,
    FsDelete,
    FsExists,
    FsSize,
    FsIsDir,
    EnvGet,
    ConfigGet,
}

impl fmt::Display for IntrinsicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FsReadText => write!(f, "FsReadText"),
            Self::FsReadLines => write!(f, "FsReadLines"),
            Self::FsWriteText => write!(f, "FsWriteText"),
            Self::FsAppendText => write!(f, "FsAppendText"),
            Self::FsDelete => write!(f, "FsDelete"),
            Self::FsExists => write!(f, "FsExists"),
            Self::FsSize => write!(f, "FsSize"),
            Self::FsIsDir => write!(f, "FsIsDir"),
            Self::EnvGet => write!(f, "EnvGet"),
            Self::ConfigGet => write!(f, "ConfigGet"),
        }
    }
}

pub type IntrinsicCall = fn(&[DolangValue], &RuntimeContext) -> Result<DolangValue, Error>;

#[derive(Debug, Clone, Default)]
pub struct IntrinsicRegistry {
    handlers: HashMap<IntrinsicId, IntrinsicCall>,
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

    pub fn register(&mut self, id: IntrinsicId, call: IntrinsicCall) {
        self.handlers.insert(id, call);
    }

    pub fn unregister(&mut self, id: IntrinsicId) {
        self.handlers.remove(&id);
    }

    pub fn call(
        &self,
        id: IntrinsicId,
        args: &[DolangValue],
        context: &RuntimeContext,
    ) -> Result<DolangValue, Error> {
        let Some(handler) = self.handlers.get(&id) else {
            return Err(Error::Interpreter(format!(
                "runtime intrinsic '{}' is not registered",
                id
            )));
        };
        handler(args, context)
    }

    fn register_defaults(&mut self) {
        self.register(IntrinsicId::FsReadText, fs_read_text);
        self.register(IntrinsicId::FsReadLines, fs_read_lines);
        self.register(IntrinsicId::FsWriteText, fs_write_text);
        self.register(IntrinsicId::FsAppendText, fs_append_text);
        self.register(IntrinsicId::FsDelete, fs_delete);
        self.register(IntrinsicId::FsExists, fs_exists);
        self.register(IntrinsicId::FsSize, fs_size);
        self.register(IntrinsicId::FsIsDir, fs_is_dir);
        self.register(IntrinsicId::EnvGet, env_get);
        self.register(IntrinsicId::ConfigGet, config_get);
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimePolicy {
    denied: HashSet<IntrinsicId>,
}

impl RuntimePolicy {
    pub fn allow_all() -> Self {
        Self::default()
    }

    pub fn deny(&mut self, id: IntrinsicId) {
        self.denied.insert(id);
    }

    pub fn allows(&self, id: IntrinsicId) -> bool {
        !self.denied.contains(&id)
    }

    pub fn check(&self, id: IntrinsicId) -> Result<(), Error> {
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

pub fn intrinsic_string_arg(
    id: IntrinsicId,
    args: &[DolangValue],
    index: usize,
) -> Result<&str, Error> {
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
    id: IntrinsicId,
    args: &[DolangValue],
    index: usize,
) -> Result<PathBuf, Error> {
    Ok(PathBuf::from(intrinsic_string_arg(id, args, index)?))
}

fn fs_read_text(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsReadText, args, 0)?;
    match fs::read_to_string(&path) {
        Ok(content) => Ok(DolangValue::Str(content)),
        Err(err) => Err(Error::Interpreter(format!("cannot read file: {}", err))),
    }
}

fn fs_read_lines(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsReadLines, args, 0)?;
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

fn fs_write_text(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsWriteText, args, 0)?;
    let content = intrinsic_string_arg(IntrinsicId::FsWriteText, args, 1)?;
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

fn fs_append_text(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsAppendText, args, 0)?;
    let content = intrinsic_string_arg(IntrinsicId::FsAppendText, args, 1)?;

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(DolangValue::Null),
            Err(err) => Err(Error::Interpreter(format!("write error: {}", err))),
        },
        Err(err) => Err(Error::Interpreter(format!("cannot open file: {}", err))),
    }
}

fn fs_delete(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsDelete, args, 0)?;
    match fs::remove_file(&path) {
        Ok(_) => Ok(DolangValue::Null),
        Err(err) => Err(Error::Interpreter(format!("cannot delete file: {}", err))),
    }
}

fn fs_exists(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsExists, args, 0)?;
    Ok(DolangValue::Bool(fs::metadata(path).is_ok()))
}

fn fs_size(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsSize, args, 0)?;
    match fs::metadata(path) {
        Ok(metadata) => Ok(DolangValue::Int(metadata.len() as i64)),
        Err(_) => Ok(DolangValue::Int(0)),
    }
}

fn fs_is_dir(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let path = intrinsic_path_buf_arg(IntrinsicId::FsIsDir, args, 0)?;
    match fs::metadata(path) {
        Ok(metadata) => Ok(DolangValue::Bool(metadata.is_dir())),
        Err(_) => Ok(DolangValue::Bool(false)),
    }
}

fn env_get(args: &[DolangValue], _context: &RuntimeContext) -> Result<DolangValue, Error> {
    let key = intrinsic_string_arg(IntrinsicId::EnvGet, args, 0)?;
    match std::env::var(key) {
        Ok(value) => Ok(DolangValue::Str(value)),
        Err(_) => Err(Error::Interpreter(format!(
            "environment variable '{}' is not defined",
            key
        ))),
    }
}

fn config_get(args: &[DolangValue], context: &RuntimeContext) -> Result<DolangValue, Error> {
    let key = intrinsic_string_arg(IntrinsicId::ConfigGet, args, 0)?;

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::module::ProjectConfig;

    use super::*;
    use super::{IntrinsicId, IntrinsicRegistry, RuntimePolicy};
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
        registry.register(IntrinsicId::FsReadText, echo);

        let context = test_context(RuntimeMode::Test);
        let value = registry
            .call(
                IntrinsicId::FsReadText,
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
            .call(IntrinsicId::FsReadText, &[], &context)
            .expect_err("missing intrinsic should fail");
        assert!(error.to_string().contains("is not registered"));
    }

    #[test]
    fn runtime_policy_can_deny_intrinsic() {
        let mut policy = RuntimePolicy::allow_all();
        policy.deny(IntrinsicId::FsReadText);

        let error = policy
            .check(IntrinsicId::FsReadText)
            .expect_err("denied intrinsic should fail");
        assert!(error.to_string().contains("denied by runtime policy"));
    }

    #[test]
    fn intrinsic_argument_validation_reports_type_errors() {
        let error = intrinsic_string_arg(IntrinsicId::FsReadText, &[DolangValue::Int(1)], 0)
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
