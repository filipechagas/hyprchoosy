//! Configuration types and loading logic for hyprchoosy.
//!
//! This module handles reading and parsing the TOML configuration file,
//! including default browser settings and routing rules.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultSection,
    #[serde(flatten)]
    pub sections: HashMap<String, RuleSection>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DefaultSection {
    #[serde(default = "default_browser")]
    pub browser: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleSection {
    pub browser: String,
    #[serde(default)]
    pub clients: Vec<String>,
    #[serde(default)]
    pub url: Vec<String>,
}

fn default_browser() -> String {
    "firefox".to_string()
}

fn xdg_config_home() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config")
}

pub fn config_path() -> PathBuf {
    if let Ok(p) = env::var("HYPRCHOOSY_CONFIG") {
        return PathBuf::from(p);
    }
    xdg_config_home().join("hyprchoosy/config.toml")
}

pub fn load_config() -> Result<Config> {
    let path = config_path();
    let data = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read config at {}. Set HYPRCHOOSY_CONFIG to override.",
            path.display()
        )
    })?;
    let cfg: Config = toml::from_str(&data).with_context(|| "Invalid TOML in config")?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[default]
browser = "firefox"

[work]
browser = "chrome"
clients = ["slack", "teams"]
url = ["company.com"]
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default.browser, "firefox");
        assert_eq!(config.sections.len(), 1);

        let work = config.sections.get("work").unwrap();
        assert_eq!(work.browser, "chrome");
        assert_eq!(work.clients.len(), 2);
        assert_eq!(work.url.len(), 1);
    }

    #[test]
    fn test_config_default_browser() {
        let toml_str = r#"
[default]

[work]
browser = "chrome"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default.browser, "firefox");
    }

    #[test]
    fn test_config_no_default_section() {
        let toml_str = r#"
[work]
browser = "chrome"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default.browser, "");
    }

    #[test]
    fn test_config_empty_lists() {
        let toml_str = r#"
[section]
browser = "chrome"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let section = config.sections.get("section").unwrap();
        assert_eq!(section.clients.len(), 0);
        assert_eq!(section.url.len(), 0);
    }

    #[test]
    fn test_xdg_config_home_with_env() {
        std::env::set_var("XDG_CONFIG_HOME", "/custom/config");
        let path = xdg_config_home();
        assert_eq!(path, PathBuf::from("/custom/config"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_config_path_with_override() {
        std::env::set_var("HYPRCHOOSY_CONFIG", "/custom/config.toml");
        let path = config_path();
        assert_eq!(path, PathBuf::from("/custom/config.toml"));
        std::env::remove_var("HYPRCHOOSY_CONFIG");
    }
}
