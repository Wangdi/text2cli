# text2cli Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform CLI tool that intercepts shell input with `@@@` trigger, calls AI agents for command suggestions, and injects results into terminal input buffer.

**Architecture:** Rust CLI with modular agent adapters, shell-specific hooks for bash/zsh/pwsh, and TOML-based configuration. Uses trait-based agent abstraction for extensibility.

**Tech Stack:** Rust, Tokio (async), Clap (CLI), Serde/TOML (config), PSReadLine integration (Windows)

---

## File Structure

```
src/
├── main.rs              # CLI entry point with clap
├── lib.rs               # Library exports
├── error.rs             # Error types
├── config.rs            # Config data structures + loader
├── parser.rs            # TriggerParser
├── context.rs           # ContextCollector
├── router.rs            # AgentRouter
├── executor.rs          # AgentExecutor
├── session.rs           # SessionManager
├── shell/
│   ├── mod.rs           # Shell module exports
│   ├── bash.rs          # Bash hook generation
│   ├── zsh.rs           # Zsh hook generation
│   └── pwsh.rs          # PowerShell hook generation
└── agents/
    ├── mod.rs           # Agent trait + registry
    ├── claude.rs        # Claude Code adapter
    ├── codex.rs         # Codex adapter
    └── generic.rs       # Generic agent adapter
shell/
├── text2cli.bash        # Bash hook script
├── text2cli.zsh         # Zsh hook script
└── text2cli.ps1         # PowerShell hook script
tests/
├── integration_tests.rs
└── fixtures/
    └── config.toml      # Test config fixture
config.example.toml      # Example configuration
```

---

## Task 1: Project Setup and Error Types

**Files:**
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/error.rs`

- [ ] **Step 1: Create main.rs with empty main function**

```rust
fn main() {
    println!("text2cli");
}
```

- [ ] **Step 2: Create lib.rs with module declarations**

```rust
pub mod error;

pub use error::{Error, Result};
```

- [ ] **Step 3: Create error.rs with custom error types**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent execution failed: {0}")]
    AgentExecution(String),

    #[error("No command returned from agent")]
    NoCommandReturned,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 4: Verify project compiles**

Run: `cargo build`
Expected: Compiles successfully with no errors

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/lib.rs src/error.rs Cargo.toml
git commit -m "feat: initial project setup with error types"
```

---

## Task 2: Configuration System

**Files:**
- Create: `src/config.rs`
- Create: `config.example.toml`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test for ConfigLoader**

Create `tests/config_test.rs`:

```rust
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
    let mut config = Config::default();
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test config_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement Config data structures**

Create `src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub enabled: bool,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_trigger")]
    pub trigger: String,

    #[serde(default = "default_agent")]
    pub default_agent: String,

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

        agents.insert("claude-code".to_string(), AgentConfig {
            enabled: true,
            command: "claude".to_string(),
        });

        agents.insert("codex".to_string(), AgentConfig {
            enabled: true,
            command: "codex".to_string(),
        });

        agents.insert("opencode".to_string(), AgentConfig {
            enabled: false,
            command: "opencode".to_string(),
        });

        Self {
            trigger: default_trigger(),
            default_agent: default_agent(),
            agents,
        }
    }
}

impl Config {
    pub fn get_enabled_agents(&self) -> impl Iterator<Item = (&String, &AgentConfig)> {
        self.agents.iter().filter(|(_, c)| c.enabled)
    }

    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        Self::load_from_path(&config_path)
    }

    pub fn config_path() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| Error::Config("Cannot determine home directory".to_string()))?;
        Ok(home.join(".text2cli").join("config.toml"))
    }

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
```

- [ ] **Step 4: Update lib.rs to include config module**

```rust
pub mod error;
pub mod config;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test config_test`
Expected: All tests pass

- [ ] **Step 6: Create example config file**

Create `config.example.toml`:

```toml
# text2cli configuration
# Copy to ~/.text2cli/config.toml

# Trigger string to detect agent commands
trigger = "@@@"

# Default agent to use when not specified
default_agent = "claude-code"

# Agent configurations
[agents.claude-code]
enabled = true
command = "claude"

[agents.codex]
enabled = true
command = "codex"

[agents.opencode]
enabled = false
command = "opencode"

[agents.cursor-cli]
enabled = false
command = "cursor-cli"

[agents.gemini]
enabled = false
command = "gemini"

[agents.openclaw]
enabled = false
command = "openclaw"

[agents.hermes]
enabled = false
command = "hermes"
```

- [ ] **Step 7: Commit**

```bash
git add src/config.rs src/lib.rs config.example.toml tests/config_test.rs
git commit -m "feat: add configuration system with TOML support"
```

---

## Task 3: Trigger Parser

**Files:**
- Create: `src/parser.rs`
- Create: `tests/parser_test.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for TriggerParser**

Create `tests/parser_test.rs`:

```rust
use text2cli::parser::{TriggerParser, ParsedCommand, ParsePosition};

#[test]
fn test_parse_prefix_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@ 重命名这个变量").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
    assert_eq!(cmd.position, ParsePosition::Prefix);
}

#[test]
fn test_parse_suffix_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("重命名这个变量 @@@").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
    assert_eq!(cmd.position, ParsePosition::Suffix);
}

#[test]
fn test_no_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("echo hello").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_empty_content() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_whitespace_handling() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@   重命名这个变量   ").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
}

#[test]
fn test_custom_trigger() {
    let parser = TriggerParser::new("!!!");

    let result = parser.parse("!!! fix this bug").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "fix this bug");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test parser_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement TriggerParser**

Create `src/parser.rs`:

```rust
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum ParsePosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub content: String,
    pub position: ParsePosition,
    pub raw_input: String,
}

pub struct TriggerParser {
    trigger: String,
}

impl TriggerParser {
    pub fn new(trigger: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<Option<ParsedCommand>> {
        let trimmed = input.trim();

        // Check for empty input
        if trimmed.is_empty() {
            return Ok(None);
        }

        // Try prefix match
        if let Some(content) = self.try_prefix(trimmed) {
            if content.is_empty() {
                return Ok(None);
            }
            return Ok(Some(ParsedCommand {
                content: content.to_string(),
                position: ParsePosition::Prefix,
                raw_input: input.to_string(),
            }));
        }

        // Try suffix match
        if let Some(content) = self.try_suffix(trimmed) {
            if content.is_empty() {
                return Ok(None);
            }
            return Ok(Some(ParsedCommand {
                content: content.to_string(),
                position: ParsePosition::Suffix,
                raw_input: input.to_string(),
            }));
        }

        Ok(None)
    }

    fn try_prefix<'a>(&self, input: &'a str) -> Option<&'a str> {
        if input.starts_with(&self.trigger) {
            let rest = input[self.trigger.len()..].trim();
            Some(rest)
        } else {
            None
        }
    }

    fn try_suffix<'a>(&self, input: &'a str) -> Option<&'a str> {
        if input.ends_with(&self.trigger) {
            let rest = input[..input.len() - self.trigger.len()].trim();
            Some(rest)
        } else {
            None
        }
    }
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod error;
pub mod config;
pub mod parser;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
pub use parser::{TriggerParser, ParsedCommand, ParsePosition};
```

- [ ] **Step 5: Run tests**

Run: `cargo test parser_test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/parser.rs src/lib.rs tests/parser_test.rs
git commit -m "feat: add trigger parser with prefix/suffix support"
```

---

## Task 4: Context Collector

**Files:**
- Create: `src/context.rs`
- Create: `tests/context_test.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for ContextCollector**

Create `tests/context_test.rs`:

```rust
use std::fs;
use tempfile::tempdir;
use text2cli::context::{ContextCollector, Context, GitStatus};

#[test]
fn test_collect_working_dir() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let context = ContextCollector::collect().unwrap();
    std::env::set_current_dir(original_dir).unwrap();

    assert!(context.working_dir.ends_with(dir.path().file_name().unwrap()));
}

#[test]
fn test_context_has_git_status_in_non_git_dir() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let context = ContextCollector::collect().unwrap();
    std::env::set_current_dir(original_dir).unwrap();

    // Not a git repo, so git_status should be None
    assert!(context.git_status.is_none());
}

#[test]
fn test_context_default() {
    let context = Context::default();
    assert!(context.git_branch.is_none());
    assert!(context.git_status.is_none());
    assert!(context.recent_files.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test context_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement ContextCollector**

Create `src/context.rs`:

```rust
use std::path::PathBuf;
use std::process::Command;
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub working_dir: PathBuf,
    pub git_branch: Option<String>,
    pub git_status: Option<GitStatus>,
    pub recent_files: Vec<PathBuf>,
    pub shell_env: std::collections::HashMap<String, String>,
}

pub struct ContextCollector;

impl ContextCollector {
    pub fn collect() -> Result<Context> {
        let working_dir = std::env::current_dir()
            .map_err(|e| Error::Parse(format!("Cannot get current directory: {}", e)))?;

        let git_branch = Self::get_git_branch(&working_dir);
        let git_status = Self::get_git_status(&working_dir)?;
        let recent_files = Self::get_recent_files(&working_dir);
        let shell_env = Self::get_shell_env();

        Ok(Context {
            working_dir,
            git_branch,
            git_status,
            recent_files,
            shell_env,
        })
    }

    fn get_git_branch(dir: &PathBuf) -> Option<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
        None
    }

    fn get_git_status(dir: &PathBuf) -> Result<Option<GitStatus>> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(dir)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let status_str = String::from_utf8_lossy(&output.stdout);
                let mut status = GitStatus::default();

                for line in status_str.lines() {
                    let prefix = line.chars().take(2).collect::<String>();
                    match prefix.trim() {
                        "M" | "MM" | " M" => status.modified += 1,
                        "A" | "AM" | " A" => status.added += 1,
                        "D" | "AD" | " D" => status.deleted += 1,
                        "??" => status.untracked += 1,
                        _ => {}
                    }
                }

                Ok(Some(status))
            }
            _ => Ok(None),
        }
    }

    fn get_recent_files(_dir: &PathBuf) -> Vec<PathBuf> {
        // TODO: Implement file watching or history tracking
        Vec::new()
    }

    fn get_shell_env() -> std::collections::HashMap<String, String> {
        std::env::vars()
            .filter(|(k, _)| k.starts_with("SHELL") || k.starts_with("TERM"))
            .collect()
    }
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod error;
pub mod config;
pub mod parser;
pub mod context;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
pub use parser::{TriggerParser, ParsedCommand, ParsePosition};
pub use context::{ContextCollector, Context, GitStatus};
```

- [ ] **Step 5: Run tests**

Run: `cargo test context_test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/context.rs src/lib.rs tests/context_test.rs
git commit -m "feat: add context collector for git and environment info"
```

---

## Task 5: Agent Trait and Adapters

**Files:**
- Create: `src/agents/mod.rs`
- Create: `src/agents/claude.rs`
- Create: `src/agents/codex.rs`
- Create: `src/agents/generic.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for AgentAdapter trait**

Create `tests/agent_test.rs`:

```rust
use text2cli::agents::{AgentAdapter, ClaudeAdapter, GenericAdapter, AgentRegistry};
use text2cli::context::Context;

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
fn test_generic_adapter() {
    let adapter = GenericAdapter::new("my-agent", "my-agent-cli");

    assert_eq!(adapter.name(), "my-agent");
    assert_eq!(adapter.command(), "my-agent-cli");
}

#[test]
fn test_agent_registry() {
    let mut registry = AgentRegistry::new();
    registry.register(Box::new(ClaudeAdapter::new("claude")));
    registry.register(Box::new(GenericAdapter::new("codex", "codex")));

    assert!(registry.get("claude").is_some());
    assert!(registry.get("codex").is_some());
    assert!(registry.get("unknown").is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test agent_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement Agent trait and adapters**

Create `src/agents/mod.rs`:

```rust
mod claude;
mod codex;
mod generic;

pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use generic::GenericAdapter;

use crate::{Error, Result};
use crate::context::Context;
use std::collections::HashMap;

/// Trait for agent adapters
pub trait AgentAdapter: Send + Sync {
    /// Get the agent name
    fn name(&self) -> &str;

    /// Get the CLI command to invoke
    fn command(&self) -> &str;

    /// Build the prompt with context
    fn build_prompt(&self, request: &str, context: &Context) -> String {
        format!(
            r#"You are a command suggestion assistant. Given the user's request and context,
respond with ONLY the shell command(s) to execute. No explanations.

Context:
- Working directory: {}

User request: {}

Respond with the command only, one per line if multiple."#,
            context.working_dir.display(),
            request
        )
    }

    /// Parse agent output into command(s)
    fn parse_output(&self, output: &str) -> Result<Vec<String>>;
}

/// Registry for agent adapters
pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn AgentAdapter>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn AgentAdapter>) {
        self.agents.insert(adapter.name().to_string(), adapter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn AgentAdapter> {
        self.agents.get(name).map(|b| b.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.agents.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(ClaudeAdapter::new("claude-code")));
        registry.register(Box::new(CodexAdapter::new("codex")));
        registry
    }
}
```

Create `src/agents/claude.rs`:

```rust
use super::{AgentAdapter, Context};
use crate::{Error, Result};

pub struct ClaudeAdapter {
    command: String,
}

impl ClaudeAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Try to extract from code blocks first
        let commands = extract_code_blocks(output);
        if !commands.is_empty() {
            return Ok(commands);
        }

        // Fall back to non-empty lines
        let commands: Vec<String> = output
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"))
            .take(5) // Limit to 5 commands
            .map(|s| s.to_string())
            .collect();

        if commands.is_empty() {
            Err(Error::NoCommandReturned)
        } else {
            Ok(commands)
        }
    }
}

fn extract_code_blocks(output: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_code_block = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block && !trimmed.is_empty() {
            commands.push(trimmed.to_string());
        }
    }

    commands
}
```

Create `src/agents/codex.rs`:

```rust
use super::{AgentAdapter, Context};
use crate::{Error, Result};

pub struct CodexAdapter {
    command: String,
}

impl CodexAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "codex"
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Try JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
            if let Some(cmd) = json.get("command").and_then(|v| v.as_str()) {
                return Ok(vec![cmd.to_string()]);
            }
        }

        // Fall back to first non-empty line
        let command = output
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty());

        match command {
            Some(cmd) => Ok(vec![cmd.to_string()]),
            None => Err(Error::NoCommandReturned),
        }
    }
}
```

Create `src/agents/generic.rs`:

```rust
use super::{AgentAdapter, Context};
use crate::{Error, Result};

pub struct GenericAdapter {
    name: String,
    command: String,
}

impl GenericAdapter {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
}

impl AgentAdapter for GenericAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Simple first-line extraction
        let command = output
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"));

        match command {
            Some(cmd) => Ok(vec![cmd.to_string()]),
            None => Err(Error::NoCommandReturned),
        }
    }
}
```

- [ ] **Step 4: Add serde_json dependency**

Update `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
anyhow = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
dirs = "5"
which = "6"
```

- [ ] **Step 5: Update lib.rs**

```rust
pub mod error;
pub mod config;
pub mod parser;
pub mod context;
pub mod agents;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
pub use parser::{TriggerParser, ParsedCommand, ParsePosition};
pub use context::{ContextCollector, Context, GitStatus};
pub use agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
```

- [ ] **Step 6: Run tests**

Run: `cargo test agent_test`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src/agents/ src/lib.rs tests/agent_test.rs Cargo.toml
git commit -m "feat: add agent adapter trait with claude, codex, generic implementations"
```

---

## Task 6: Agent Executor

**Files:**
- Create: `src/executor.rs`
- Create: `tests/executor_test.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for AgentExecutor**

Create `tests/executor_test.rs`:

```rust
use text2cli::executor::AgentExecutor;
use text2cli::agents::GenericAdapter;
use text2cli::context::Context;

#[test]
fn test_executor_build_command() {
    let adapter = GenericAdapter::new("test", "echo");
    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // This is a basic test - actual execution tests need mocking
    assert!(executor.adapter().name() == "test");
}

#[tokio::test]
async fn test_executor_with_echo_command() {
    let adapter = GenericAdapter::new("test", "echo");
    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // Echo should just return the prompt
    let result = executor.execute("hello world").await;

    // Should succeed since echo is a valid command
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test executor_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement AgentExecutor**

Create `src/executor.rs`:

```rust
use crate::agents::AgentAdapter;
use crate::context::Context;
use crate::{Error, Result};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

pub struct AgentExecutor {
    adapter: Box<dyn AgentAdapter>,
    context: Context,
}

impl AgentExecutor {
    pub fn new(adapter: Box<dyn AgentAdapter>, context: Context) -> Self {
        Self { adapter, context }
    }

    pub fn adapter(&self) -> &dyn AgentAdapter {
        self.adapter.as_ref()
    }

    pub async fn execute(&self, request: &str) -> Result<Vec<String>> {
        let prompt = self.adapter.build_prompt(request, &self.context);
        let output = self.run_agent(&prompt).await?;
        self.adapter.parse_output(&output)
    }

    async fn run_agent(&self, prompt: &str) -> Result<String> {
        let mut child = Command::new(self.adapter.command())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::AgentNotFound(self.adapter.command().to_string())
                } else {
                    Error::AgentExecution(format!("Failed to spawn agent: {}", e))
                }
            })?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        // Read stdout
        let mut stdout = String::new();
        if let Some(mut stdout_handle) = child.stdout.take() {
            stdout_handle.read_to_string(&mut stdout).await?;
        }

        // Check exit status
        let status = child.wait().await?;
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut stderr_handle) = child.stderr.take() {
                stderr_handle.read_to_string(&mut stderr).await?;
            }
            return Err(Error::AgentExecution(format!(
                "Agent exited with {}: {}",
                status, stderr
            )));
        }

        Ok(stdout)
    }
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod error;
pub mod config;
pub mod parser;
pub mod context;
pub mod agents;
pub mod executor;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
pub use parser::{TriggerParser, ParsedCommand, ParsePosition};
pub use context::{ContextCollector, Context, GitStatus};
pub use agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
pub use executor::AgentExecutor;
```

- [ ] **Step 5: Run tests**

Run: `cargo test executor_test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/executor.rs src/lib.rs tests/executor_test.rs
git commit -m "feat: add agent executor with async process spawning"
```

---

## Task 7: Shell Integration - PowerShell

**Files:**
- Create: `src/shell/mod.rs`
- Create: `src/shell/pwsh.rs`
- Create: `shell/text2cli.ps1`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for PowerShell hook**

Create `tests/shell_test.rs`:

```rust
use text2cli::shell::{PwshHook, ShellHook};

#[test]
fn test_pwsh_hook_generate() {
    let hook = PwshHook::new("text2cli");
    let script = hook.generate();

    assert!(script.contains("text2cli"));
    assert!(script.contains("PSReadLine"));
}

#[test]
fn test_pwsh_hook_trigger_detection() {
    let hook = PwshHook::new("text2cli");
    let trigger = "@@@";

    assert!(hook.should_intercept("@@@ test", trigger));
    assert!(!hook.should_intercept("echo hello", trigger));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test shell_test --no-run`
Expected: Compilation error - module not found

- [ ] **Step 3: Implement ShellHook trait and PwshHook**

Create `src/shell/mod.rs`:

```rust
mod pwsh;
mod bash;
mod zsh;

pub use pwsh::PwshHook;
pub use bash::BashHook;
pub use zsh::ZshHook;

pub trait ShellHook {
    /// Shell name
    fn name(&self) -> &str;

    /// Generate hook script
    fn generate(&self) -> String;

    /// Check if input should be intercepted
    fn should_intercept(&self, input: &str, trigger: &str) -> bool;
}
```

Create `src/shell/pwsh.rs`:

```rust
use super::ShellHook;

pub struct PwshHook {
    binary_name: String,
}

impl PwshHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for PwshHook {
    fn name(&self) -> &str {
        "pwsh"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli PowerShell integration
# Add to $PROFILE

function Invoke-Text2Cli {{
    param([string]$Input)

    if ($Input -match '^@@@') {{
        $result = {} process $Input
        if ($result) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($result)
        }}
    }}
}}

# Register as a command validation handler
Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {{
    param($key, $arg)

    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    if ($line -match '^@@@') {{
        $result = {} process $line
        if ($result) {{
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            return
        }}
    }}

    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}}
"#,
            self.binary_name, self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
```

Create `src/shell/bash.rs`:

```rust
use super::ShellHook;

pub struct BashHook {
    binary_name: String,
}

impl BashHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for BashHook {
    fn name(&self) -> &str {
        "bash"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli bash integration
# Add to ~/.bashrc

__text2cli_preexec__() {{
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({} process "$cmd")
        if [[ -n "$result" ]]; then
            READLINE_LINE="$result"
        fi
    fi
}}

preexec_functions+=(__text2cli_preexec__)
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
```

Create `src/shell/zsh.rs`:

```rust
use super::ShellHook;

pub struct ZshHook {
    binary_name: String,
}

impl ZshHook {
    pub fn new(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

impl ShellHook for ZshHook {
    fn name(&self) -> &str {
        "zsh"
    }

    fn generate(&self) -> String {
        format!(
            r#"
# text2cli zsh integration
# Add to ~/.zshrc

__text2cli_preexec__() {{
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$({} process "$cmd")
        if [[ -n "$result" ]]; then
            print -z "$result"
        fi
    fi
}}

preexec_functions+=(__text2cli_preexec__)
"#,
            self.binary_name
        )
    }

    fn should_intercept(&self, input: &str, trigger: &str) -> bool {
        input.trim().starts_with(trigger)
    }
}
```

- [ ] **Step 4: Create standalone PowerShell script**

Create `shell/text2cli.ps1`:

```powershell
# text2cli PowerShell Integration
# Source this file or add to $PROFILE

function Invoke-Text2Cli {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Input
    )

    $trigger = "@@@"
    if ($Input -match "^$trigger") {
        $content = $Input.Substring($trigger.Length).Trim()
        $result = text2cli process $content
        if ($result) {
            return $result
        }
    }
    return $Input
}

# Key handler for Enter key with trigger detection
Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
    param($key, $arg)

    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    if ($line -match '^@@@') {
        $result = text2cli process $line
        if ($result) {
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $result)
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            return
        }
    }

    [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
}

Write-Host "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
```

- [ ] **Step 5: Update lib.rs**

```rust
pub mod error;
pub mod config;
pub mod parser;
pub mod context;
pub mod agents;
pub mod executor;
pub mod shell;

pub use error::{Error, Result};
pub use config::{Config, ConfigLoader, AgentConfig};
pub use parser::{TriggerParser, ParsedCommand, ParsePosition};
pub use context::{ContextCollector, Context, GitStatus};
pub use agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
pub use executor::AgentExecutor;
pub use shell::{ShellHook, PwshHook, BashHook, ZshHook};
```

- [ ] **Step 6: Run tests**

Run: `cargo test shell_test`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src/shell/ src/lib.rs shell/text2cli.ps1 tests/shell_test.rs
git commit -m "feat: add shell integration for PowerShell, Bash, and Zsh"
```

---

## Task 8: CLI Entry Point

**Files:**
- Modify: `src/main.rs`
- Create: `tests/cli_test.rs`

- [ ] **Step 1: Write failing CLI tests**

Create `tests/cli_test.rs`:

```rust
use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .output_contains("text2cli");
}

#[test]
fn test_cli_init_pwsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "pwsh"])
        .assert()
        .success()
        .output_contains("PSReadLine");
}

#[test]
fn test_cli_init_bash() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "bash"])
        .assert()
        .success()
        .output_contains("preexec");
}

#[test]
fn test_cli_init_zsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "zsh"])
        .assert()
        .success()
        .output_contains("print -z");
}

#[test]
fn test_cli_list_agents() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["list-agents"])
        .assert()
        .success()
        .output_contains("claude-code");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test cli_test --no-run`
Expected: Compilation errors

- [ ] **Step 3: Implement CLI with clap**

Update `src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use text2cli::{
    Config, ConfigLoader, ContextCollector, AgentExecutor, AgentRegistry,
    ClaudeAdapter, CodexAdapter, GenericAdapter, PwshHook, BashHook, ZshHook, ShellHook,
};
use text2cli::parser::TriggerParser;

#[derive(Parser)]
#[command(name = "text2cli")]
#[command(about = "AI-powered command suggestion CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input to process (if no subcommand)
    #[arg(trailing_var_arg = true)]
    input: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize shell integration
    Init {
        /// Shell type: bash, zsh, pwsh
        shell: String,
    },

    /// Process input and return command
    Process {
        /// Input to process
        #[arg(trailing_var_arg = true)]
        input: Vec<String>,
    },

    /// List available agents
    ListAgents,

    /// Show configuration
    Config,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = ConfigLoader::load().unwrap_or_else(|e| {
        eprintln!("[text2cli] Warning: {}", e);
        Config::default()
    });

    match cli.command {
        Some(Commands::Init { shell }) => {
            handle_init(&shell);
        }
        Some(Commands::Process { input }) => {
            handle_process(&input, &config).await;
        }
        Some(Commands::ListAgents) => {
            handle_list_agents(&config);
        }
        Some(Commands::Config) => {
            handle_config();
        }
        None => {
            if !cli.input.is_empty() {
                handle_process(&cli.input, &config).await;
            } else {
                println!("text2cli - AI-powered command suggestion CLI");
                println!("Run 'text2cli --help' for usage.");
            }
        }
    }
}

fn handle_init(shell: &str) {
    match shell {
        "bash" => {
            let hook = BashHook::new("text2cli");
            println!("{}", hook.generate());
        }
        "zsh" => {
            let hook = ZshHook::new("text2cli");
            println!("{}", hook.generate());
        }
        "pwsh" | "powershell" => {
            let hook = PwshHook::new("text2cli");
            println!("{}", hook.generate());
        }
        _ => {
            eprintln!("[text2cli] Unknown shell: {}", shell);
            eprintln!("Supported shells: bash, zsh, pwsh");
            std::process::exit(1);
        }
    }
}

async fn handle_process(input: &[String], config: &Config) {
    let input_str = input.join(" ");
    let parser = TriggerParser::new(&config.trigger);

    let parsed = match parser.parse(&input_str) {
        Ok(Some(cmd)) => cmd,
        Ok(None) => {
            // No trigger found, pass through
            println!("{}", input_str);
            return;
        }
        Err(e) => {
            eprintln!("[text2cli] Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let context = match ContextCollector::collect() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("[text2cli] Context error: {}", e);
            std::process::exit(1);
        }
    };

    let mut registry = AgentRegistry::default();
    registry.register(Box::new(GenericAdapter::new("gemini", "gemini")));
    registry.register(Box::new(GenericAdapter::new("openclaw", "openclaw")));
    registry.register(Box::new(GenericAdapter::new("hermes", "hermes")));

    let agent = registry.get(&config.default_agent).unwrap_or_else(|| {
        eprintln!("[text2cli] Agent '{}' not found", config.default_agent);
        std::process::exit(1);
    });

    let executor = AgentExecutor::new(
        Box::new(ClaudeAdapter::new(agent.command())),
        context,
    );

    match executor.execute(&parsed.content).await {
        Ok(commands) => {
            // Output first command for injection
            println!("{}", commands[0]);
        }
        Err(e) => {
            eprintln!("[text2cli] Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_list_agents(config: &Config) {
    println!("Available agents:");
    for (name, agent_config) in &config.agents {
        let status = if agent_config.enabled { "enabled" } else { "disabled" };
        println!("  {} ({}) - {}", name, agent_config.command, status);
    }
}

fn handle_config() {
    match ConfigLoader::config_path() {
        Ok(path) => {
            println!("Config path: {}", path.display());
            match ConfigLoader::load() {
                Ok(config) => {
                    println!("Trigger: {}", config.trigger);
                    println!("Default agent: {}", config.default_agent);
                }
                Err(e) => {
                    println!("Using defaults: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[text2cli] {}", e);
        }
    }
}
```

- [ ] **Step 4: Add assert_cmd output assertion helper**

The tests use `output_contains` which isn't built-in. Update tests to use predicates:

Update `tests/cli_test.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("text2cli"));
}

#[test]
fn test_cli_init_pwsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "pwsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PSReadLine"));
}

#[test]
fn test_cli_init_bash() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("preexec"));
}

#[test]
fn test_cli_init_zsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("print -z"));
}

#[test]
fn test_cli_list_agents() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["list-agents"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-code"));
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test cli_test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/cli_test.rs
git commit -m "feat: add CLI entry point with init, process, list-agents commands"
```

---

## Task 9: Shell Scripts for Bash and Zsh

**Files:**
- Create: `shell/text2cli.bash`
- Create: `shell/text2cli.zsh`

- [ ] **Step 1: Create Bash integration script**

Create `shell/text2cli.bash`:

```bash
# text2cli Bash Integration
# Add to ~/.bashrc: source /path/to/text2cli.bash

__text2cli_preexec__() {
    local cmd="$BASH_COMMAND"
    if [[ "$cmd" == @@@* ]]; then
        local result=$(text2cli process "$cmd" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            # For bash, we need to use bind -x for buffer manipulation
            # This is a simplified version that echoes the command
            echo "$result"
        fi
    fi
}

# Use PROMPT_COMMAND for pre-execution hook
__text2cli_prompt_cmd__() {
    __text2cli_last_cmd=$BASH_COMMAND
}

# Alternative: use bind for Enter key
bind -x '"\C-m": "__text2cli_enter"'

__text2cli_enter() {
    local line="${READLINE_LINE:-}"
    if [[ "$line" == @@@* ]]; then
        local result=$(text2cli process "$line" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            READLINE_LINE="$result"
            READLINE_POINT=${#READLINE_LINE}
        else
            builtin accept-line
        fi
    else
        builtin accept-line
    fi
}

echo "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
```

- [ ] **Step 2: Create Zsh integration script**

Create `shell/text2cli.zsh`:

```zsh
# text2cli Zsh Integration
# Add to ~/.zshrc: source /path/to/text2cli.zsh

__text2cli_preexec__() {
    local cmd="$1"
    if [[ "$cmd" == @@@* ]]; then
        local result=$(text2cli process "$cmd" 2>/dev/null)
        if [[ -n "$result" && $? -eq 0 ]]; then
            # Use print -z to inject into input buffer
            print -z "$result"
            # Return non-zero to cancel original command
            return 1
        fi
    fi
    return 0
}

# Add to preexec_functions
preexec_functions+=(__text2cli_preexec__)

echo "text2cli loaded. Use '@@@ <instruction>' to get command suggestions."
```

- [ ] **Step 3: Commit**

```bash
git add shell/text2cli.bash shell/text2cli.zsh
git commit -m "feat: add standalone bash and zsh integration scripts"
```

---

## Task 10: Test Fixtures

**Files:**
- Create: `tests/fixtures/config.toml`

- [ ] **Step 1: Create test fixture**

Create `tests/fixtures/config.toml`:

```toml
trigger = "@@@"
default_agent = "claude-code"

[agents.claude-code]
enabled = true
command = "claude"

[agents.codex]
enabled = true
command = "codex"

[agents.test-agent]
enabled = true
command = "echo"
```

- [ ] **Step 2: Add integration test using fixture**

Create `tests/integration_tests.rs`:

```rust
use std::path::PathBuf;
use text2cli::{Config, ConfigLoader};

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
```

- [ ] **Step 3: Update lib.rs to expose load_from_path**

The function is already public. Run tests:

Run: `cargo test integration_tests`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add tests/fixtures/config.toml tests/integration_tests.rs
git commit -m "feat: add test fixtures and integration tests"
```

---

## Task 11: Final Build and Documentation

**Files:**
- Create: `README.md`
- Create: `.gitignore`

- [ ] **Step 1: Create .gitignore**

Create `.gitignore`:

```
/target
/Cargo.lock
/.idea
/.vscode
*.swp
*.swo
*~
.DS_Store
```

- [ ] **Step 2: Create README.md**

Create `README.md`:

```markdown
# text2cli

AI-powered command suggestion CLI that integrates with your shell.

## Installation

```bash
cargo install --path .
```

## Shell Integration

### PowerShell

Add to your `$PROFILE`:

```powershell
. /path/to/shell/text2cli.ps1
```

Or use the init command:

```powershell
Invoke-Expression (text2cli init pwsh)
```

### Bash

Add to `~/.bashrc`:

```bash
source /path/to/shell/text2cli.bash
```

Or:

```bash
eval "$(text2cli init bash)"
```

### Zsh

Add to `~/.zshrc`:

```zsh
source /path/to/shell/text2cli.zsh
```

Or:

```zsh
eval "$(text2cli init zsh)"
```

## Usage

Type `@@@` followed by your instruction:

```
$ @@@ 重命名这个变量
$ git mv old_name new_name  # Command injected, press Enter to execute
```

## Configuration

Configuration file: `~/.text2cli/config.toml`

```toml
trigger = "@@@"
default_agent = "claude-code"

[agents.claude-code]
enabled = true
command = "claude"

[agents.codex]
enabled = true
command = "codex"
```

## Supported Agents

- claude-code (Anthropic)
- codex (OpenAI)
- opencode
- cursor-cli
- gemini
- And more via generic adapter

## License

MIT
```

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 4: Build release**

Run: `cargo build --release`
Expected: Build succeeds

- [ ] **Step 5: Final commit**

```bash
git add README.md .gitignore
git commit -m "docs: add README and gitignore"
```

---

## Summary

**Files Created:**
- `src/main.rs` - CLI entry point
- `src/lib.rs` - Library exports
- `src/error.rs` - Error types
- `src/config.rs` - Configuration system
- `src/parser.rs` - Trigger parser
- `src/context.rs` - Context collector
- `src/executor.rs` - Agent executor
- `src/agents/mod.rs` - Agent trait and registry
- `src/agents/claude.rs` - Claude adapter
- `src/agents/codex.rs` - Codex adapter
- `src/agents/generic.rs` - Generic adapter
- `src/shell/mod.rs` - Shell hook trait
- `src/shell/pwsh.rs` - PowerShell hook
- `src/shell/bash.rs` - Bash hook
- `src/shell/zsh.rs` - Zsh hook
- `shell/text2cli.ps1` - PowerShell script
- `shell/text2cli.bash` - Bash script
- `shell/text2cli.zsh` - Zsh script
- `config.example.toml` - Example config
- `tests/fixtures/config.toml` - Test fixture
- `tests/config_test.rs` - Config tests
- `tests/parser_test.rs` - Parser tests
- `tests/context_test.rs` - Context tests
- `tests/agent_test.rs` - Agent tests
- `tests/executor_test.rs` - Executor tests
- `tests/shell_test.rs` - Shell tests
- `tests/cli_test.rs` - CLI tests
- `tests/integration_tests.rs` - Integration tests
- `README.md` - Documentation
- `.gitignore` - Git ignore

**Total: 30 files**
