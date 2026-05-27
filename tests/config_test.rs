use std::fs;
use tempfile::tempdir;
use text2cli::{AgentConfig, Config, ConfigLoader};

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

    fs::write(
        &config_path,
        r#"
trigger = "!!!"
default_agent = "codex"

[agents.codex]
enabled = true
command = "codex"
"#,
    )
    .unwrap();

    let config = ConfigLoader::load_from_path(&config_path).unwrap();

    assert_eq!(config.trigger, "!!!");
    assert_eq!(config.default_agent, "codex");
    assert!(config.agents.contains_key("codex"));
}

#[test]
fn test_get_enabled_agents() {
    let mut config = Config::default();

    // Override: disable codex, add a disabled agent
    config.agents.insert(
        "codex".to_string(),
        AgentConfig {
            enabled: false,
            command: "codex".to_string(),
        },
    );
    config.agents.insert(
        "disabled-agent".to_string(),
        AgentConfig {
            enabled: false,
            command: "disabled".to_string(),
        },
    );

    let enabled: Vec<_> = config.get_enabled_agents().collect();
    // Only claude-code should be enabled now
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

// === Merge behavior tests ===

#[test]
fn test_partial_config_merges_defaults() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // User only specifies trigger, not agents
    fs::write(&config_path, r#"trigger = "!!!""#).unwrap();

    let config = ConfigLoader::load_from_path(&config_path).unwrap();

    // Trigger should be user-specified
    assert_eq!(config.trigger, "!!!");
    // Agents should be merged from defaults
    assert!(config.agents.contains_key("claude-code"));
    assert!(config.agents.contains_key("codex"));
    assert!(config.agents.contains_key("opencode"));
}

#[test]
fn test_custom_agent_preserves_defaults() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // User adds a custom agent
    fs::write(
        &config_path,
        r#"
[agents.my-custom-agent]
enabled = true
command = "my-agent"
"#,
    )
    .unwrap();

    let config = ConfigLoader::load_from_path(&config_path).unwrap();

    // Custom agent should be present
    assert!(config.agents.contains_key("my-custom-agent"));
    // Default agents should still be present
    assert!(config.agents.contains_key("claude-code"));
    assert!(config.agents.contains_key("codex"));
}

// === Validation tests ===

#[test]
fn test_validate_empty_trigger_fails() {
    let mut config = Config::default();
    config.trigger = "   ".to_string();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Trigger string cannot be empty"));
}

#[test]
fn test_validate_missing_default_agent_fails() {
    let mut config = Config::default();
    config.default_agent = "nonexistent".to_string();
    config.agents.remove("nonexistent");

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("is not defined in agents map"));
}

#[test]
fn test_validate_valid_config_passes() {
    let config = Config::default();
    assert!(config.validate().is_ok());
}

// === Error path tests ===

#[test]
fn test_invalid_toml_fails() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    fs::write(&config_path, "this is not valid toml [[[").unwrap();

    let result = ConfigLoader::load_from_path(&config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse config"));
}

#[test]
fn test_invalid_trigger_in_file_fails() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // Empty trigger should fail validation
    fs::write(&config_path, r#"trigger = """#).unwrap();

    let result = ConfigLoader::load_from_path(&config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Trigger string cannot be empty"));
}

#[test]
fn test_missing_default_agent_in_file_fails() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // Specify default_agent that doesn't exist
    fs::write(
        &config_path,
        r#"
default_agent = "nonexistent"
"#,
    )
    .unwrap();

    let result = ConfigLoader::load_from_path(&config_path);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("is not defined in agents map"));
}

#[test]
fn test_config_path_returns_valid_path() {
    let path = ConfigLoader::config_path().unwrap();
    assert!(path.to_string_lossy().contains(".text2cli"));
    assert!(path.to_string_lossy().contains("config.toml"));
}
