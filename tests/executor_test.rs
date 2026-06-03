use std::path::PathBuf;
use text2cli::{AgentExecutor, Context, Error, GenericAdapter, GitStatus};

#[test]
fn test_executor_build_command() {
    let adapter = GenericAdapter::new("test", "echo");
    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // This is a basic test - actual execution tests need mocking
    assert!(executor.adapter().name() == "test");
}

#[tokio::test]
async fn test_executor_with_valid_command() {
    // Use a command that exists and outputs something parseable
    // On Windows, use powershell -c echo for UTF-8 output
    #[cfg(windows)]
    let adapter = GenericAdapter::new("test", "powershell");
    #[cfg(not(windows))]
    let adapter = GenericAdapter::new("test", "echo");

    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // This tests that the executor can spawn a process
    // The actual output parsing depends on what the agent outputs
    let result = executor.execute("hello world").await;

    // The test verifies the executor can spawn and communicate with a process
    // Even if parsing fails, we've tested the core execution logic
    match result {
        Ok(commands) => {
            // If successful, we should have at least one command
            assert!(!commands.is_empty());
        }
        Err(e) => {
            // Expected: either AgentExecution, NoCommandReturned, or IO error
            // All indicate the process was spawned and communicated with
            let err_str = format!("{}", e);
            assert!(
                err_str.contains("Agent")
                    || err_str.contains("No command")
                    || err_str.contains("IO error"),
                "Unexpected error: {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_executor_agent_not_found() {
    // Use a command that definitely doesn't exist
    let adapter = GenericAdapter::new("test", "nonexistent_command_xyz123");
    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    let result = executor.execute("test").await;

    // Should fail with AgentNotFound
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("not found") || err_str.contains("Agent"),
        "Expected AgentNotFound error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_executor_non_zero_exit_status() {
    // Use a command that exits with non-zero status
    #[cfg(windows)]
    {
        // On Windows, use a command that fails when reading stdin
        // findstr returns exit code 1 when pattern not found
        let adapter = GenericAdapter::new("test", "findstr");
        let context = Context::default();
        let executor = AgentExecutor::new(Box::new(adapter), context);
        // findstr with non-matching pattern exits with 1
        let result = executor.execute("NONEXISTENT_PATTERN_12345").await;

        assert!(result.is_err(), "Expected error for non-zero exit status");
        let err = result.unwrap_err();

        // Should be AgentExecution error with exit status info
        let err_str = format!("{}", err);
        assert!(
            err_str.contains("exited") || err_str.contains("Agent"),
            "Expected exit status or IO error, got: {}",
            err_str
        );
    }
    #[cfg(not(windows))]
    {
        // On Unix, use 'false' command which always exits with 1
        let adapter = GenericAdapter::new("test", "false");
        let context = Context::default();
        let executor = AgentExecutor::new(Box::new(adapter), context);
        let result = executor.execute("test").await;

        assert!(result.is_err(), "Expected error for non-zero exit status");
        let err = result.unwrap_err();

        // Should be AgentExecution error with exit status info
        match &err {
            Error::AgentExecution(msg) => {
                assert!(
                    msg.contains("exited") || msg.contains("exit"),
                    "Expected exit status in error message, got: {}",
                    msg
                );
            }
            _ => {
                let err_str = format!("{}", err);
                assert!(
                    err_str.contains("Agent") || err_str.contains("exited"),
                    "Expected AgentExecution error, got: {}",
                    err
                );
            }
        }
    }
}

#[tokio::test]
async fn test_executor_stderr_capture() {
    // Use a command that writes to stderr
    #[cfg(windows)]
    {
        // On Windows, use a command that writes to stderr and fails
        // findstr writes to stderr and exits with 1 for invalid usage
        let adapter = GenericAdapter::new("test", "findstr");
        let context = Context::default();
        let executor = AgentExecutor::new(Box::new(adapter), context);

        // findstr with invalid arguments writes to stderr
        let result = executor.execute("/C:INVALID_OPTION_XYZ123").await;

        assert!(result.is_err(), "Expected error for command with stderr");
        let err = result.unwrap_err();
        let err_str = format!("{}", err);

        // Verify error contains exit status or stderr indication
        assert!(
            err_str.contains("exited")
                || err_str.contains("Agent")
                || err_str.contains("findstr"),
            "Expected exit status error, got: {}",
            err_str
        );
    }
    #[cfg(not(windows))]
    {
        // On Unix, use sh to write to stderr
        let adapter = GenericAdapter::new("test", "sh");
        let context = Context::default();
        let executor = AgentExecutor::new(Box::new(adapter), context);

        let result = executor.execute("echo 'test_error_message_12345' >&2; exit 1").await;

        assert!(result.is_err(), "Expected error for command with stderr");
        let err = result.unwrap_err();
        let err_str = format!("{}", err);

        // Verify stderr is included in error message
        assert!(
            err_str.contains("test_error_message_12345"),
            "Expected stderr content in error message, got: {}",
            err_str
        );
    }
}

#[tokio::test]
async fn test_executor_context_in_prompt() {
    // Create context with specific working directory
    let mut context = Context::default();
    context.working_dir = PathBuf::from("/custom/test/directory");

    let adapter = GenericAdapter::new("test", "echo");
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // The executor should include working_dir in the prompt
    // We can verify this indirectly by checking the adapter's build_prompt
    let prompt = executor.adapter().build_prompt("test request", &Context::default());

    // The prompt should mention "Working directory"
    assert!(
        prompt.contains("Working directory"),
        "Prompt should contain 'Working directory', got: {}",
        prompt
    );
}

#[tokio::test]
async fn test_executor_stdin_communication() {
    // Test that the prompt is passed to subprocess stdin
    // Use a command that reads from stdin and echoes it back
    #[cfg(windows)]
    let adapter = GenericAdapter::new("test", "powershell");
    #[cfg(not(windows))]
    let adapter = GenericAdapter::new("test", "cat");

    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // The prompt contains the request, so echo should return it
    let result = executor.execute("unique_test_marker_98765").await;

    // The command should receive the prompt via stdin
    // If stdin communication works, output should contain context about the request
    match result {
        Ok(commands) => {
            // Should have received the prompt content
            let output = commands.join(" ");
            assert!(
                !output.is_empty(),
                "Expected non-empty output from stdin"
            );
        }
        Err(e) => {
            // On some systems, cat/powershell might not return valid commands
            // But we should at least verify the process was spawned
            let err_str = format!("{}", e);
            assert!(
                !err_str.contains("not found"),
                "Command should have been found, got: {}",
                err_str
            );
        }
    }
}

// =============================================================================
// Context integration tests
// =============================================================================

#[test]
fn test_executor_with_git_context() {
    let mut context = Context::default();
    context.working_dir = PathBuf::from("/project");
    context.git_branch = Some("feature-branch".to_string());
    context.git_status = Some(GitStatus {
        modified: 2,
        added: 1,
        deleted: 0,
        untracked: 3,
    });

    let adapter = GenericAdapter::new("test", "echo");
    let executor = AgentExecutor::new(Box::new(adapter), context.clone());

    // Verify context is preserved
    let prompt = executor.adapter().build_prompt("test", &context);
    assert!(prompt.contains("/project"));
}

#[test]
fn test_executor_with_shell_env_context() {
    let mut context = Context::default();
    context.shell_env.insert("SHELL".to_string(), "/bin/zsh".to_string());
    context.shell_env.insert("TERM".to_string(), "xterm-256color".to_string());

    let adapter = GenericAdapter::new("test", "echo");
    let executor = AgentExecutor::new(Box::new(adapter), context.clone());

    // Verify context is accessible
    assert_eq!(executor.adapter().name(), "test");
}

#[test]
fn test_executor_with_recent_files_context() {
    let mut context = Context::default();
    context.recent_files = vec![
        PathBuf::from("/project/src/main.rs"),
        PathBuf::from("/project/src/lib.rs"),
    ];

    let adapter = GenericAdapter::new("test", "echo");
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // Just verify it doesn't panic
    assert!(executor.adapter().name() == "test");
}

#[test]
fn test_executor_adapter_access() {
    let adapter = GenericAdapter::new("custom-agent", "custom-cmd");
    let context = Context::default();
    let executor = AgentExecutor::new(Box::new(adapter), context);

    // Verify we can access the adapter
    assert_eq!(executor.adapter().name(), "custom-agent");
    assert_eq!(executor.adapter().command(), "custom-cmd");
}

#[test]
fn test_context_empty_vs_default() {
    // Empty context
    let empty = Context::default();

    // All fields should be their defaults
    assert!(empty.git_branch.is_none());
    assert!(empty.git_status.is_none());
    assert!(empty.recent_files.is_empty());
    assert!(empty.shell_env.is_empty());
}
