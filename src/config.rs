use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{Error, Result};

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
    "@@@".to_string()
}

fn default_agent() -> String {
    "claude-code".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let mut agents = HashMap::new();

        agents.insert(
            "claude-code".to_string(),
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

        Self {
            trigger: default_trigger(),
            default_agent: default_agent(),
            agents,
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

        Ok(config)
    }
}
