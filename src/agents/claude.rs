use super::AgentAdapter;
use crate::context::Context;
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

    /// Build prompt optimized for Claude Code print mode
    fn build_claude_prompt(&self, request: &str, context: &Context) -> String {
        format!(
            r#"You are a command suggestion assistant. Respond with ONLY shell command(s).
No explanations, no markdown formatting, no code blocks.

Context:
- Working directory: {}
- Git branch: {}
- Modified files: {}

User request: {}

Output the command directly. If multiple commands, one per line."#,
            context.working_dir.display(),
            context.git_branch.as_deref().unwrap_or("unknown"),
            context.git_status.as_ref().map(|s| s.modified.to_string()).unwrap_or_else(|| "unknown".to_string()),
            request
        )
    }

    /// Parse Claude Code print mode JSON output
    fn parse_json_output(&self, output: &str) -> Option<Vec<String>> {
        let json: serde_json::Value = serde_json::from_str(output).ok()?;

        // Claude Code -p returns {"result": "...", "type": "result", ...}
        if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
            return Some(self.parse_text_commands(result));
        }

        // Alternative format: {"commands": [...]}
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

    /// Parse text-based command output
    fn parse_text_commands(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|l| l.trim())
            .filter(|l| {
                !l.is_empty()
                    && !l.starts_with('#')
                    && !l.starts_with("//")
                    && !l.starts_with("```")
                    && !l.to_lowercase().starts_with("here is")
                    && !l.to_lowercase().starts_with("the command")
            })
            .take(5) // Limit to 5 commands
            .map(|s| s.to_string())
            .collect()
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn build_prompt(&self, request: &str, context: &Context) -> String {
        self.build_claude_prompt(request, context)
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Try JSON format first (Claude Code -p output)
        if let Some(commands) = self.parse_json_output(output) {
            if !commands.is_empty() {
                return Ok(commands);
            }
        }

        // Try to extract from code blocks
        let commands = extract_code_blocks(output);
        if !commands.is_empty() {
            return Ok(commands);
        }

        // Fall back to text parsing
        let commands = self.parse_text_commands(output);
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
    let mut code_block_start = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Detect code block start/end
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
                code_block_start = true;
            }
            continue;
        }

        // Collect commands from inside code blocks
        if in_code_block && !trimmed.is_empty() {
            // Skip language identifier on first line
            if code_block_start && (trimmed == "bash" || trimmed == "sh" || trimmed == "shell") {
                code_block_start = false;
                continue;
            }
            code_block_start = false;
            commands.push(trimmed.to_string());
        }
    }

    commands
}
