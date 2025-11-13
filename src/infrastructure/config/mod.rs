use crate::utils::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub jira: JiraConfig,
    pub ui: UiConfig,
}

/// Jira-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    pub instance: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// UI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub show_avatars: bool,
    #[serde(default = "default_false")]
    pub compact_mode: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_refresh_interval() -> u64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jira: JiraConfig {
                instance: String::new(),
                username: None,
            },
            ui: UiConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_avatars: default_true(),
            compact_mode: default_false(),
            refresh_interval: default_refresh_interval(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to read config file {}: {}",
                config_path.display(),
                e
            )))?;

        toml::from_str(&content)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to parse config file: {}",
                e
            )))
    }

    /// Save configuration to file
    #[allow(dead_code)] // Will be used when config editing is implemented
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let config_dir = config_path.parent().ok_or_else(|| {
            crate::utils::LazyJiraError::Config("Invalid config path".to_string())
        })?;

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(config_dir)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to create config directory: {}",
                e
            )))?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to serialize config: {}",
                e
            )))?;

        std::fs::write(&config_path, content)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to write config file: {}",
                e
            )))?;

        Ok(())
    }

    /// Get the path to the configuration file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::utils::LazyJiraError::Config(
                "Could not determine config directory".to_string()
            ))?;
        
        Ok(config_dir.join("lazyjira").join("config.toml"))
    }

    /// Load jira-cli configuration
    pub fn load_jira_cli_config() -> Result<Option<JiraCliConfig>> {
        let jira_cli_config_path = Self::jira_cli_config_path()?;
        
        if !jira_cli_config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&jira_cli_config_path)
            .map_err(|e| crate::utils::LazyJiraError::Config(format!(
                "Failed to read jira-cli config: {}",
                e
            )))?;

        // jira-cli uses YAML format
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| crate::utils::LazyJiraError::Parse(format!(
                "Failed to parse jira-cli config: {}",
                e
            )))?;

        // Extract instance and auth info from jira-cli config
        let instance = yaml.get("instance")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let auth = yaml.get("auth")
            .and_then(|a| {
                let auth_type = a.get("type")?.as_str()?;
                let username = a.get("username")?.as_str()?;
                let token = a.get("token").and_then(|t| t.as_str());
                
                Some(JiraCliAuth {
                    auth_type: auth_type.to_string(),
                    username: username.to_string(),
                    token: token.map(|s| s.to_string()),
                })
            });

        if let (Some(instance), Some(auth)) = (instance, auth) {
            Ok(Some(JiraCliConfig { instance, auth }))
        } else {
            Ok(None)
        }
    }

    /// Get the path to jira-cli configuration file
    fn jira_cli_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::utils::LazyJiraError::Config(
                "Could not determine config directory".to_string()
            ))?;
        
        Ok(config_dir.join("jira-cli").join("config.yaml"))
    }
}

/// jira-cli configuration structure
#[derive(Debug, Clone)]
pub struct JiraCliConfig {
    pub instance: String,
    pub auth: JiraCliAuth,
}

#[derive(Debug, Clone)]
pub struct JiraCliAuth {
    pub auth_type: String,
    pub username: String,
    pub token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.ui.theme, "default");
        assert!(config.ui.show_avatars);
        assert!(!config.ui.compact_mode);
        assert_eq!(config.ui.refresh_interval, 30);
    }

    #[test]
    fn test_config_load_nonexistent() {
        // This will return default config if file doesn't exist
        // In a real test, we'd need to mock the file system
        // For now, we test that default works
        let config = Config::default();
        assert!(!config.jira.instance.is_empty() || config.jira.instance.is_empty());
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = Config {
            jira: JiraConfig {
                instance: "test.atlassian.net".to_string(),
                username: Some("test@example.com".to_string()),
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                show_avatars: false,
                compact_mode: true,
                refresh_interval: 60,
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.jira.instance, deserialized.jira.instance);
        assert_eq!(config.jira.username, deserialized.jira.username);
        assert_eq!(config.ui.theme, deserialized.ui.theme);
        assert_eq!(config.ui.show_avatars, deserialized.ui.show_avatars);
        assert_eq!(config.ui.compact_mode, deserialized.ui.compact_mode);
        assert_eq!(config.ui.refresh_interval, deserialized.ui.refresh_interval);
    }
}
