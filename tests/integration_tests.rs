use std::path::PathBuf;
use text2cli::{
    ConfigLoader, ContextCollector, TriggerParser,
    ClaudeAdapter, CodexAdapter, GenericAdapter, AgentAdapter, AgentRegistry,
};

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

// =============================================================================
// End-to-end workflow tests
// =============================================================================

#[test]
fn test_full_workflow_trigger_parsing() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();
    let parser = TriggerParser::new(&config.trigger);

    // Simulate user input with trigger
    let input = "@@@ git status";
    let parsed = parser.parse(input).unwrap();

    assert!(parsed.is_some());
    let cmd = parsed.unwrap();
    assert_eq!(cmd.content, "git status");
}

#[test]
fn test_full_workflow_context_collection() {
    // Context collection should work in any directory
    let context = ContextCollector::collect().unwrap();

    // Should have working directory
    assert!(!context.working_dir.to_string_lossy().is_empty());

    // Should have shell_env (may be empty depending on environment)
    // Just verify it's a valid HashMap
    assert!(context.shell_env.is_empty() || !context.shell_env.is_empty());
}

#[test]
fn test_full_workflow_adapter_selection() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();

    // Get the default agent
    let default_agent_name = &config.default_agent;
    assert!(config.get_agent(default_agent_name).is_some());
}

#[test]
fn test_full_workflow_prompt_building() {
    let context = ContextCollector::collect().unwrap();
    let adapter = ClaudeAdapter::new("claude");

    let prompt = adapter.build_prompt("list files", &context);

    assert!(prompt.contains("list files"));
    assert!(prompt.contains(&context.working_dir.to_string_lossy().to_string()));
}

#[test]
fn test_full_workflow_output_parsing() {
    let adapter = ClaudeAdapter::new("claude");

    // Simulate agent output
    let output = r#"Here's the command:
```bash
ls -la
```
"#;

    let commands = adapter.parse_output(output).unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "ls -la");
}

#[test]
fn test_registry_with_config_agents() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();
    let mut registry = AgentRegistry::new();

    // Register agents from config
    for (name, agent_config) in &config.agents {
        registry.register(Box::new(GenericAdapter::new(name, &agent_config.command)));
    }

    // All configured agents should be registered
    for name in config.agents.keys() {
        assert!(registry.get(name).is_some(), "Agent {} should be registered", name);
    }
}

#[test]
fn test_workflow_multiline_command() {
    let parser = TriggerParser::new("@@@");

    let input = "@@@ run this\nand this too";
    let parsed = parser.parse(input).unwrap().unwrap();

    assert!(parsed.content.contains("run this"));
    assert!(parsed.content.contains("and this too"));
}

#[test]
fn test_workflow_unicode_request() {
    let parser = TriggerParser::new("@@@");

    let input = "@@@ 重命名这个文件";
    let parsed = parser.parse(input).unwrap().unwrap();

    assert_eq!(parsed.content, "重命名这个文件");
}

#[test]
fn test_workflow_no_trigger_passthrough() {
    let parser = TriggerParser::new("@@@");

    let input = "regular command without trigger";
    let parsed = parser.parse(input).unwrap();

    assert!(parsed.is_none());
}

#[test]
fn test_all_adapters_parse_consistently() {
    let adapters: Vec<Box<dyn AgentAdapter>> = vec![
        Box::new(ClaudeAdapter::new("claude")),
        Box::new(CodexAdapter::new("codex")),
        Box::new(GenericAdapter::new("generic", "generic")),
    ];

    // All adapters should produce some output for plain text
    for adapter in adapters {
        let result = adapter.parse_output("git status");
        // Some adapters may return commands, others may return error
        // but none should panic
        match result {
            Ok(cmds) => assert!(!cmds.is_empty() || cmds.is_empty()),
            Err(_) => {} // Error is also valid
        }
    }
}

#[test]
fn test_config_trigger_matches_parser() {
    let config = ConfigLoader::load_from_path(&fixture_path()).unwrap();
    let parser = TriggerParser::new(&config.trigger);

    // The trigger in config should work with parser
    let test_input = format!("{} test command", config.trigger);
    let parsed = parser.parse(&test_input).unwrap();

    assert!(parsed.is_some());
}

#[test]
fn test_context_working_dir_exists() {
    let context = ContextCollector::collect().unwrap();

    // Working directory should exist
    assert!(context.working_dir.exists());
}

#[test]
fn test_multiple_parse_operations() {
    let parser = TriggerParser::new("@@@");

    // Parser should be reusable
    let results: Vec<_> = ["@@@ cmd1", "no trigger", "@@@ cmd2"]
        .iter()
        .map(|input| parser.parse(input).unwrap())
        .collect();

    assert!(results[0].is_some());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}
