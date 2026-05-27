use std::io;
use text2cli::error::{Error, Result};

#[test]
fn test_config_error_display() {
    let error = Error::Config("test message".to_string());
    assert_eq!(format!("{}", error), "Configuration error: test message");
}

#[test]
fn test_agent_not_found_error_display() {
    let error = Error::AgentNotFound("claude".to_string());
    assert_eq!(format!("{}", error), "Agent not found: claude");
}

#[test]
fn test_agent_execution_error_display() {
    let error = Error::AgentExecution("failed".to_string());
    assert_eq!(format!("{}", error), "Agent execution failed: failed");
}

#[test]
fn test_no_command_returned_error_display() {
    let error = Error::NoCommandReturned;
    assert_eq!(format!("{}", error), "No command returned from agent");
}

#[test]
fn test_parse_error_display() {
    let error = Error::Parse("invalid".to_string());
    assert_eq!(format!("{}", error), "Parse error: invalid");
}

#[test]
fn test_io_error_from_conversion() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error: Error = io_error.into();

    match error {
        Error::Io(e) => {
            assert!(e.contains("file not found"));
        }
        _ => panic!("Expected Error::Io variant"),
    }
}

#[test]
fn test_io_error_display() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let error: Error = io_error.into();

    let display = format!("{}", error);
    assert!(display.starts_with("IO error:"));
    assert!(display.contains("access denied"));
}

#[test]
fn test_error_propagation() -> Result<()> {
    fn inner_function() -> Result<()> {
        Err(Error::Config("inner failure".to_string()))
    }

    fn outer_function() -> Result<()> {
        inner_function()?;
        Ok(())
    }

    let result = outer_function();

    match result {
        Err(Error::Config(msg)) => assert_eq!(msg, "inner failure"),
        _ => panic!("Expected Config error to propagate"),
    }

    Ok(())
}

#[test]
fn test_result_type_alias() {
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }

    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_chained_error_propagation() -> Result<()> {
    fn level_three() -> Result<i32> {
        Err(Error::Parse("failed at level three".to_string()))
    }

    fn level_two() -> Result<i32> {
        let value = level_three()?;
        Ok(value + 1)
    }

    fn level_one() -> Result<i32> {
        let value = level_two()?;
        Ok(value + 1)
    }

    let result = level_one();

    match result {
        Err(Error::Parse(msg)) => assert_eq!(msg, "failed at level three"),
        _ => panic!("Expected Parse error to propagate through chain"),
    }

    Ok(())
}

#[test]
fn test_io_error_automatic_conversion() {
    fn io_operation() -> Result<()> {
        let io_error = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        Err(io_error.into())
    }

    let result = io_operation();

    match result {
        Err(Error::Io(e)) => assert!(e.contains("pipe broken")),
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_debug_trait() {
    let error = Error::Config("test".to_string());
    let debug_output = format!("{:?}", error);
    assert!(debug_output.contains("Config"));
}

#[test]
fn test_error_from_io_error_trait() {
    // Test that the From trait is implemented correctly
    let io_error = io::Error::new(io::ErrorKind::TimedOut, "timeout");
    let error = Error::from(io_error);

    match error {
        Error::Io(e) => assert!(e.contains("timeout")),
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_json_error() {
    let error = Error::Json("parse error".to_string());
    assert_eq!(format!("{}", error), "JSON error: parse error");
}

#[test]
fn test_json_error_from_serde_json() {
    let json_str = "{invalid json";
    let result: Result<()> = serde_json::from_str(json_str).map_err(|e| e.into());
    
    match result {
        Err(Error::Json(_)) => {}
        _ => panic!("Expected Json error"),
    }
}
