use super::AgentAdapter;
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
        let command = output.lines().map(|l| l.trim()).find(|l| !l.is_empty());

        match command {
            Some(cmd) => Ok(vec![cmd.to_string()]),
            None => Err(Error::NoCommandReturned),
        }
    }
}
