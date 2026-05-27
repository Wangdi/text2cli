use text2cli::agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
use text2cli::context::Context;
use text2cli::Error;

#[test]
fn test_claude_adapter_build_prompt() {
    let adapter = ClaudeAdapter::new("claude");
    let context = Context::default();

    let prompt = adapter.build_prompt("重命名这个变量", &context);

    assert!(prompt.contains("重命名这个变量"));
    assert!(prompt.contains("command suggestion"));
}

#[test]
fn test_claude_adapter_parse_plain_command() {
    let adapter = ClaudeAdapter::new("claude");

    let output = "git mv old_name new_name";
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "git mv old_name new_name");
}

#[test]
fn test_claude_adapter_parse_code_block() {
    let adapter = ClaudeAdapter::new("claude");

    let output = r#"Here's the command:
```bash
git mv old_name new_name
```
"#;
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "git mv old_name new_name");
}

#[test]
fn test_claude_adapter_parse_multiple_code_blocks() {
    let adapter = ClaudeAdapter::new("claude");

    let output = r#"Here are the commands:
```bash
git add .
git commit -m "message"
```
"#;
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0], "git add .");
    assert_eq!(commands[1], r#"git commit -m "message""#);
}

#[test]
fn test_claude_adapter_parse_filters_comments() {
    let adapter = ClaudeAdapter::new("claude");

    let output = r#"# This is a comment
git status
// Another comment
git log
"#;
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0], "git status");
    assert_eq!(commands[1], "git log");
}

#[test]
fn test_claude_adapter_parse_empty_returns_error() {
    let adapter = ClaudeAdapter::new("claude");

    let result = adapter.parse_output("");
    assert!(matches!(result, Err(Error::NoCommandReturned)));

    let result = adapter.parse_output("# only comments");
    assert!(matches!(result, Err(Error::NoCommandReturned)));
}

#[test]
fn test_claude_adapter_limits_commands() {
    let adapter = ClaudeAdapter::new("claude");

    let output = "cmd1\ncmd2\ncmd3\ncmd4\ncmd5\ncmd6\ncmd7";
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 5); // Limited to 5 commands
}

#[test]
fn test_codex_adapter_parse_json() {
    let adapter = CodexAdapter::new("codex");

    let output = r#"{"command": "git status"}"#;
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "git status");
}

#[test]
fn test_codex_adapter_parse_plain_text() {
    let adapter = CodexAdapter::new("codex");

    let output = "git push origin main";
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "git push origin main");
}

#[test]
fn test_codex_adapter_parse_empty_returns_error() {
    let adapter = CodexAdapter::new("codex");

    let result = adapter.parse_output("");
    assert!(matches!(result, Err(Error::NoCommandReturned)));
}

#[test]
fn test_generic_adapter() {
    let adapter = GenericAdapter::new("my-agent", "my-agent-cli");

    assert_eq!(adapter.name(), "my-agent");
    assert_eq!(adapter.command(), "my-agent-cli");
}

#[test]
fn test_generic_adapter_parse() {
    let adapter = GenericAdapter::new("test", "test-cli");

    let commands = adapter.parse_output("npm install").unwrap();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "npm install");
}

#[test]
fn test_generic_adapter_parse_filters_comments() {
    let adapter = GenericAdapter::new("test", "test-cli");

    let output = "# comment\nnpm test";
    let commands = adapter.parse_output(output).unwrap();

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "npm test");
}

#[test]
fn test_generic_adapter_parse_empty_returns_error() {
    let adapter = GenericAdapter::new("test", "test-cli");

    let result = adapter.parse_output("");
    assert!(matches!(result, Err(Error::NoCommandReturned)));
}

#[test]
fn test_agent_registry() {
    let mut registry = AgentRegistry::new();
    registry.register(Box::new(ClaudeAdapter::new("claude")));
    registry.register(Box::new(GenericAdapter::new("codex", "codex")));

    assert!(registry.get("claude-code").is_some());
    assert!(registry.get("codex").is_some());
    assert!(registry.get("unknown").is_none());
}

#[test]
fn test_agent_registry_default() {
    let registry = AgentRegistry::default();

    // Default registry should have claude-code and codex
    assert!(registry.get("claude-code").is_some());
    assert!(registry.get("codex").is_some());
}

#[test]
fn test_agent_registry_list() {
    let mut registry = AgentRegistry::new();
    registry.register(Box::new(ClaudeAdapter::new("claude")));
    registry.register(Box::new(GenericAdapter::new("codex", "codex")));

    let agents = registry.list();
    assert_eq!(agents.len(), 2);
    assert!(agents.contains(&"claude-code"));
    assert!(agents.contains(&"codex"));
}
