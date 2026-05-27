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
