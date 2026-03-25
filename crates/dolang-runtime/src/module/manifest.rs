use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub env: HashMap<String, String>,
    pub server: ServerConfig,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
            entry: "main.dol".to_string(),
            env: HashMap::new(),
            server: ServerConfig::default(),
            dependencies: HashMap::new(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
        }
    }
}

impl ProjectConfig {
    pub fn load_from_dir(dir: &Path) -> Option<Self> {
        let config_path = super::paths::resolve_manifest_path(dir);
        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&config_path).ok()?;
        Self::parse_toml(&content)
    }

    pub fn parse_toml(content: &str) -> Option<Self> {
        let parsed = toml::from_str::<ProjectConfig>(content).ok()?;
        if parsed.name.is_empty() {
            return None;
        }
        Some(parsed)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|s| s.as_str())
    }
}
