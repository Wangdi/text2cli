use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{Error, Result};

// Constants for default values - no magic strings
pub const DEFAULT_TRIGGER: &str = "@@@";
pub const DEFAULT_AGENT: &str = "claude-code";

/// Configuration for an individual agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Whether this agent is enabled
    pub enabled: bool,
    /// Command to invoke the agent
    pub command: String,
}

/// Main configuration for text2cli
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Trigger string to detect agent commands
    #[serde(default = "default_trigger")]
    pub trigger: String,

    /// Default agent to use when not specified
    #[serde(default = "default_agent")]
    pub default_agent: String,

    /// Agent configurations
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
}

fn default_trigger() -> String {
    DEFAULT_TRIGGER.to_string()
}

fn default_agent() -> String {
    DEFAULT_AGENT.to_string()
}

/// Returns the default agents HashMap
fn default_agents() -> HashMap<String, AgentConfig> {
    let mut agents = HashMap::new();

    agents.insert(
        DEFAULT_AGENT.to_string(),
        AgentConfig {
            enabled: true,
            command: "claude".to_string(),
        },
    );

    agents.insert(
        "codex".to_string(),
        AgentConfig {
            enabled: true,
            command: "codex".to_string(),
        },
    );

    agents.insert(
        "opencode".to_string(),
        AgentConfig {
            enabled: false,
            command: "opencode".to_string(),
        },
    );

    agents
}

impl Default for Config {
    fn default() -> Self {
        Self {
            trigger: default_trigger(),
            default_agent: default_agent(),
            agents: default_agents(),
        }
    }
}

impl Config {
    /// Get all enabled agents
    pub fn get_enabled_agents(&self) -> impl Iterator<Item = (&String, &AgentConfig)> {
        self.agents.iter().filter(|(_, c)| c.enabled)
    }

    /// Get a specific agent configuration
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate trigger is not empty
        if self.trigger.trim().is_empty() {
            return Err(Error::Config(
                "Trigger string cannot be empty".to_string(),
            ));
        }

        // Validate default_agent exists in agents map
        if !self.agents.contains_key(&self.default_agent) {
            return Err(Error::Config(format!(
                "Default agent '{}' is not defined in agents map",
                self.default_agent
            )));
        }

        Ok(())
    }

    /// Merge with default config, preserving user-specified values
    /// while filling in missing agents from defaults
    pub fn merge_with_defaults(mut self) -> Self {
        let default_agents = default_agents();

        // Merge agents: user config takes precedence, but missing agents get defaults
        for (name, agent) in default_agents {
            self.agents.entry(name).or_insert(agent);
        }

        self
    }
}

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from default path (~/.text2cli/config.toml)
    pub fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        Self::load_from_path(&config_path)
    }

    /// Get the default configuration path
    pub fn config_path() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| Error::Config("Cannot determine home directory".to_string()))?;
        Ok(home.join(".text2cli").join("config.toml"))
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &Path) -> Result<Config> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;

        // Merge with defaults and validate
        let config = config.merge_with_defaults();
        config.validate()?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_not_empty() {
        assert!(!DEFAULT_TRIGGER.is_empty());
        assert!(!DEFAULT_AGENT.is_empty());
    }

    #[test]
    fn test_default_agents_not_empty() {
        let agents = default_agents();
        assert!(!agents.is_empty());
        assert!(agents.contains_key(DEFAULT_AGENT));
    }
}
