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
