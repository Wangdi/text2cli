use crate::agents::AgentAdapter;
use crate::context::Context;
use crate::{Error, Result};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

/// Configuration for agent execution
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Timeout for agent execution
    pub timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Delay between retries
    pub retry_delay: Duration,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 2,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl ExecutorConfig {
    /// Create config with custom timeout
    pub fn with_timeout(secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(secs),
            ..Default::default()
        }
    }

    /// Create config with custom retry settings
    pub fn with_retry(max_retries: u32, delay_secs: u64) -> Self {
        Self {
            max_retries,
            retry_delay: Duration::from_secs(delay_secs),
            ..Default::default()
        }
    }
}

pub struct AgentExecutor {
    adapter: Box<dyn AgentAdapter>,
    context: Context,
    config: ExecutorConfig,
}

impl AgentExecutor {
    pub fn new(adapter: Box<dyn AgentAdapter>, context: Context) -> Self {
        Self {
            adapter,
            context,
            config: ExecutorConfig::default(),
        }
    }

    /// Create executor with custom configuration
    pub fn with_config(adapter: Box<dyn AgentAdapter>, context: Context, config: ExecutorConfig) -> Self {
        Self {
            adapter,
            context,
            config,
        }
    }

    pub fn adapter(&self) -> &dyn AgentAdapter {
        self.adapter.as_ref()
    }

    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    /// Execute with automatic retry on failure
    pub async fn execute(&self, request: &str) -> Result<Vec<String>> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.execute_once(request).await {
                Ok(commands) => return Ok(commands),
                Err(e) => {
                    last_error = Some(e.clone());

                    // Don't retry on certain errors
                    match &e {
                        Error::AgentNotFound(_) | Error::NoCommandReturned => {
                            return Err(e);
                        }
                        _ => {}
                    }

                    // Wait before retry (except on last attempt)
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(Error::AgentExecution("All retries failed".to_string())))
    }

    /// Execute once without retry
    async fn execute_once(&self, request: &str) -> Result<Vec<String>> {
        let prompt = self.adapter.build_prompt(request, &self.context);

        let output = timeout(self.config.timeout, self.run_agent(&prompt))
            .await
            .map_err(|_| {
                Error::AgentExecution(format!(
                    "Agent '{}' timed out after {:?}",
                    self.adapter.name(),
                    self.config.timeout
                ))
            })??;

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

            // Provide helpful error message
            let error_msg = if stderr.is_empty() {
                format!("Agent exited with status: {}", status)
            } else {
                format!("Agent exited with {}: {}", status, stderr)
            };

            return Err(Error::AgentExecution(error_msg));
        }

        Ok(stdout)
    }
}
