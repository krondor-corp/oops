use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct OopsConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default, rename = "rules")]
    pub rules: Vec<RuleConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct GeneralConfig {
    pub scan_depth: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleConfig {
    pub name: String,
    pub enabled: Option<bool>,
    pub dir: Option<String>,
    pub path: Option<String>,
    pub file_glob: Option<String>,
    pub when_parent_has: Option<String>,
    pub min_size: Option<String>,
    pub clean: Option<String>,
    pub description: Option<String>,
}

pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("oops")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("oops")
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

impl OopsConfig {
    pub fn load() -> Result<Self, crate::Error> {
        Self::load_from(&config_path())
    }

    pub fn load_from(path: &Path) -> Result<Self, crate::Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(crate::Error::Io)?;
        toml::from_str(&content).map_err(|e| crate::Error::ConfigParse(e.to_string()))
    }

    pub fn user_rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn disabled_builtins(&self) -> Vec<&str> {
        self.rules
            .iter()
            .filter(|r| r.enabled == Some(false))
            .map(|r| r.name.as_str())
            .collect()
    }
}

pub const DEFAULT_CONFIG_CONTENT: &str = r#"# oops configuration
# Location: ~/.config/oops/config.toml

[general]
# scan_depth = 6

# Rules define what `oops sweep` looks for.
# Built-in rules are active by default — run `oops config` to see them.
#
# To disable a built-in rule:
#   [[rules]]
#   name = "node_modules"
#   enabled = false
#
# To add a custom rule:
#   [[rules]]
#   name = "my-build-output"
#   dir = "dist"
#   when_parent_has = "package.json"
#   min_size = "100 MB"
#   clean = "rm -rf {path}"
#   description = "Frontend build output"
#
# Match modes (use exactly one per rule):
#   dir = "name"              Match directories by basename
#   path = "~/some/path"      Match a specific absolute path (~ expanded)
#   file_glob = "*.log"       Match files by extension
#
# Template variables for clean commands:
#   {path}    Full path of the matched entry
#   {parent}  Parent directory
#   {name}    Entry basename
"#;
