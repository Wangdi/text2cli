use std::path::PathBuf;
use text2cli::ConfigLoader;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("config.toml")
}

#[test]
fn test_load_fixture_config() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    assert_eq!(config.trigger, "@@@");
    assert_eq!(config.default_agent, "claude-code");
    assert!(config.agents.contains_key("test-agent"));

    let test_agent = config.get_agent("test-agent").unwrap();
    assert!(test_agent.enabled);
    assert_eq!(test_agent.command, "echo");
}

#[test]
fn test_fixture_config_has_all_agents() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    // Check explicitly defined agents
    assert!(config.agents.contains_key("claude-code"));
    assert!(config.agents.contains_key("codex"));
    assert!(config.agents.contains_key("test-agent"));
}

#[test]
fn test_fixture_config_agent_states() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    // Check claude-code is enabled
    let claude = config.get_agent("claude-code").unwrap();
    assert!(claude.enabled);
    assert_eq!(claude.command, "claude");

    // Check codex is enabled
    let codex = config.get_agent("codex").unwrap();
    assert!(codex.enabled);
    assert_eq!(codex.command, "codex");
}

#[test]
fn test_fixture_config_validates() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    // Should not panic - validates successfully
    assert!(config.validate().is_ok());
}

#[test]
fn test_fixture_config_get_enabled_agents() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    let enabled: Vec<_> = config.get_enabled_agents().collect();
    assert!(!enabled.is_empty());

    // All returned agents should be enabled
    for (_, agent_config) in enabled {
        assert!(agent_config.enabled);
    }
}
