// Project configuration module.
// Handles loading and accessing package.toml for serve mode.
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// Project configuration loaded from package.toml
#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Entry point file (default: main.dol)
    pub entry: String,
    /// Environment variables from config
    pub env: HashMap<String, String>,
    /// Server configuration
    pub server: ServerConfig,
}

/// Server configuration
#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Server host
    pub host: String,
}

impl ProjectConfig {
    /// Load project config from a directory
    pub fn load_from_dir(dir: &Path) -> Option<Self> {
        let config_path = dir.join("package.toml");
        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&config_path).ok()?;
        Self::parse_toml(&content)
    }

    /// Parse package.toml content
    pub fn parse_toml(content: &str) -> Option<Self> {
        let mut config = ProjectConfig::default();
        config.entry = "main.dol".to_string();
        config.server.port = 8080;
        config.server.host = "0.0.0.0".to_string();

        // Simple TOML parsing (only supports basic key = value pairs)
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "name" => config.name = value.to_string(),
                    "version" => config.version = value.to_string(),
                    "entry" => config.entry = value.to_string(),
                    "port" => {
                        if let Ok(p) = value.parse::<u16>() {
                            config.server.port = p;
                        }
                    }
                    "host" => config.server.host = value.to_string(),
                    _ => {
                        // Treat as environment variable
                        config.env.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        if config.name.is_empty() {
            return None;
        }

        Some(config)
    }

    /// Get a config value by key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|s| s.as_str())
    }
}

/// Global serve mode state using Mutex for thread safety
static SERVE_CONFIG: Mutex<Option<ProjectConfig>> = Mutex::new(None);

/// Check if running in serve mode
pub fn is_serve_mode() -> bool {
    SERVE_CONFIG.lock().map(|g| g.is_some()).unwrap_or(false)
}

/// Get the serve config
pub fn get_serve_config() -> Option<ProjectConfig> {
    SERVE_CONFIG.lock().ok().and_then(|g| g.clone())
}

/// Set the serve config (called at startup)
pub fn set_serve_config(config: ProjectConfig) {
    if let Ok(mut guard) = SERVE_CONFIG.lock() {
        *guard = Some(config);
    }
}
