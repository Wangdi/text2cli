use super::AgentAdapter;
use crate::context::Context;
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

    /// Build prompt for Codex
    fn build_codex_prompt(&self, request: &str, context: &Context) -> String {
        format!(
            r#"Suggest a shell command for this request. Output ONLY the command.

Context:
- Directory: {}
- Git branch: {}

Request: {}

Output the command directly without explanation."#,
            context.working_dir.display(),
            context.git_branch.as_deref().unwrap_or("unknown"),
            request
        )
    }

    /// Parse Codex JSON output
    fn parse_json_output(&self, output: &str) -> Option<Vec<String>> {
        let json: serde_json::Value = serde_json::from_str(output).ok()?;

        // Codex CLI format: {"command": "..."} or {"suggestion": "..."}
        for field in &["command", "suggestion", "cmd", "output"] {
            if let Some(cmd) = json.get(field).and_then(|v| v.as_str()) {
                return Some(vec![cmd.to_string()]);
            }
        }

        // Array format: {"commands": [...]}
        if let Some(commands) = json.get("commands").and_then(|c| c.as_array()) {
            let cmds: Vec<String> = commands
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !cmds.is_empty() {
                return Some(cmds);
            }
        }

        None
    }
}

impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "codex"
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn build_prompt(&self, request: &str, context: &Context) -> String {
        self.build_codex_prompt(request, context)
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Try JSON first
        if let Some(commands) = self.parse_json_output(output) {
            if !commands.is_empty() {
                return Ok(commands);
            }
        }

        // Extract from code blocks if present
        let commands = extract_code_blocks(output);
        if !commands.is_empty() {
            return Ok(commands);
        }

        // Fall back to first non-empty line
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

fn extract_code_blocks(output: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_code_block = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block && !trimmed.is_empty() && !trimmed.starts_with("bash") && !trimmed.starts_with("sh") {
            commands.push(trimmed.to_string());
        }
    }

    commands
}
