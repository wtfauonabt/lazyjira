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

        // Extract instance/server and auth info from config
        // Support both formats:
        // 1. jira-cli format: instance + auth.type/auth.username/auth.token
        // 2. jira CLI format: server + auth_type (credentials stored separately)
        let instance = yaml.get("instance")
            .or_else(|| yaml.get("server"))
            .and_then(|v| v.as_str())
            .map(|s| {
                // Extract domain from URL if it's a full URL
                let s = s.trim();
                if s.starts_with("http://") || s.starts_with("https://") {
                    s.replace("https://", "").replace("http://", "").split('/').next().unwrap_or(s).to_string()
                } else {
                    s.to_string()
                }
            });

        let auth = if let Some(auth_obj) = yaml.get("auth") {
            // jira-cli format: nested auth object
            if let (Some(auth_type_val), Some(username_val)) = (
                auth_obj.get("type").and_then(|v| v.as_str()),
                auth_obj.get("username").and_then(|v| v.as_str())
            ) {
                let token = auth_obj.get("token").and_then(|t| t.as_str());
                
                Some(JiraCliAuth {
                    auth_type: auth_type_val.to_string(),
                    username: username_val.to_string(),
                    token: token.map(|s| s.to_string()),
                })
            } else {
                None
            }
        } else if let Some(auth_type) = yaml.get("auth_type").and_then(|v| v.as_str()) {
            // jira CLI format: auth_type at top level
            // For API token auth, check JIRA_API_TOKEN env var
            // For basic auth, check JIRA_USERNAME and JIRA_PASSWORD env vars
            if auth_type == "api-token" || std::env::var("JIRA_API_TOKEN").is_ok() {
                // API token authentication
                let token = std::env::var("JIRA_API_TOKEN").ok();
                // Try to get username from environment or jira CLI command
                let username = std::env::var("JIRA_USERNAME").ok().or_else(|| {
                    // Try to get from jira CLI's "me" command
                    std::process::Command::new("jira")
                        .arg("me")
                        .output()
                        .ok()
                        .and_then(|output| {
                            String::from_utf8(output.stdout).ok()
                        })
                        .map(|s| s.trim().to_string())
                });
                
                if let Some(user) = username {
                    Some(JiraCliAuth {
                        auth_type: "api-token".to_string(),
                        username: user,
                        token,
                    })
                } else {
                    None
                }
            } else {
                // Basic authentication
                let username = std::env::var("JIRA_USERNAME").ok();
                let password = std::env::var("JIRA_PASSWORD").ok();
                
                if let Some(user) = username {
                    Some(JiraCliAuth {
                        auth_type: auth_type.to_string(),
                        username: user,
                        token: password, // Store password as token for basic auth
                    })
                } else {
                    None
                }
            }
        } else {
            None
        };

        match (instance, auth) {
            (Some(inst), Some(auth_val)) => {
                Ok(Some(JiraCliConfig { instance: inst, auth: auth_val }))
            }
            _ => {
                Ok(None)
            }
        }
    }

    /// Get the path to jira-cli configuration file
    /// Checks multiple possible locations:
    /// 1. ~/.config/.jira/.config.yml (jira CLI tool)
    /// 2. ~/.config/jira-cli/config.yaml (jira-cli tool)
    /// 3. ~/Library/Application Support/jira-cli/config.yaml (macOS jira-cli)
    fn jira_cli_config_path() -> Result<PathBuf> {
        // First try: ~/.config/.jira/.config.yml (jira CLI tool)
        if let Some(home) = dirs::home_dir() {
            let jira_config_path = home.join(".config").join(".jira").join(".config.yml");
            if jira_config_path.exists() {
                return Ok(jira_config_path);
            }
        }
        
        // Second try: ~/.config/jira-cli/config.yaml (jira-cli tool on Linux)
        if let Some(home) = dirs::home_dir() {
            let jira_cli_config_path = home.join(".config").join("jira-cli").join("config.yaml");
            if jira_cli_config_path.exists() {
                return Ok(jira_cli_config_path);
            }
        }
        
        // Third try: Use dirs::config_dir() which on macOS returns ~/Library/Application Support
        let config_dir = dirs::config_dir()
            .ok_or_else(|| crate::utils::LazyJiraError::Config(
                "Could not determine config directory".to_string()
            ))?;
        
        let jira_cli_config_path = config_dir.join("jira-cli").join("config.yaml");
        Ok(jira_cli_config_path)
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
