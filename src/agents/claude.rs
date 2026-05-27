use super::AgentAdapter;
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
