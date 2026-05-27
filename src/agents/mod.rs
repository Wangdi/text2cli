mod claude;
mod codex;
mod generic;

pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use generic::GenericAdapter;

use crate::context::Context;
use crate::Result;
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
        registry.register(Box::new(ClaudeAdapter::new("claude")));
        registry.register(Box::new(CodexAdapter::new("codex")));
        registry
    }
}
