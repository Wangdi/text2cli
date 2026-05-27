use std::fs;
use tempfile::tempdir;
use text2cli::config::{Config, ConfigLoader, AgentConfig};

#[test]
fn test_load_default_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // No config file should return defaults
    let config = ConfigLoader::load_from_path(&config_path).unwrap();

    assert_eq!(config.trigger, "@@@");
    assert_eq!(config.default_agent, "claude-code");
}

#[test]
fn test_load_custom_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    fs::write(&config_path, r#"
trigger = "!!!"
default_agent = "codex"

[agents.codex]
enabled = true
command = "codex"
"#).unwrap();

    let config = ConfigLoader::load_from_path(&config_path).unwrap();

    assert_eq!(config.trigger, "!!!");
    assert_eq!(config.default_agent, "codex");
    assert!(config.agents.contains_key("codex"));
}

#[test]
fn test_get_enabled_agents() {
    let mut config = Config {
        trigger: "@@@".to_string(),
        default_agent: "claude-code".to_string(),
        agents: std::collections::HashMap::new(),
    };

    config.agents.insert("claude-code".to_string(), AgentConfig {
        enabled: true,
        command: "claude".to_string(),
    });
    config.agents.insert("disabled-agent".to_string(), AgentConfig {
        enabled: false,
        command: "disabled".to_string(),
    });

    let enabled: Vec<_> = config.get_enabled_agents().collect();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].0, "claude-code");
}

#[test]
fn test_default_config_has_enabled_agents() {
    let config = Config::default();

    // Default config should have 2 enabled agents
    let enabled: Vec<_> = config.get_enabled_agents().collect();
    assert_eq!(enabled.len(), 2);

    // Verify specific agents
    assert!(config.get_agent("claude-code").is_some());
    assert!(config.get_agent("claude-code").unwrap().enabled);
    assert!(config.get_agent("codex").is_some());
    assert!(config.get_agent("codex").unwrap().enabled);

    // opencode should be disabled by default
    assert!(config.get_agent("opencode").is_some());
    assert!(!config.get_agent("opencode").unwrap().enabled);
}
