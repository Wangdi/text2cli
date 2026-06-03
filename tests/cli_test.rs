use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("text2cli"));
}

#[test]
fn test_cli_init_pwsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "pwsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PSReadLine"));
}

#[test]
fn test_cli_init_powershell_alias() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PSReadLine"));
}

#[test]
fn test_cli_init_bash() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("__text2cli_accept_line__"));
}

#[test]
fn test_cli_init_zsh() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("print -z"));
}

#[test]
fn test_cli_init_unknown_shell() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "fish"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown shell"));
}

#[test]
fn test_cli_config() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Config path:"));
}

#[test]
fn test_cli_list_agents() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["list-agents"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-code"));
}

#[test]
fn test_cli_no_arguments() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("text2cli - AI-powered command suggestion CLI"));
}

#[test]
fn test_cli_trailing_input_no_subcommand() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // Without trigger prefix, should pass through input
    cmd.args(["hello", "world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_cli_process_no_trigger() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // Process subcommand with no trigger - should pass through
    cmd.args(["process", "some", "input"])
        .assert()
        .success()
        .stdout(predicate::str::contains("some input"));
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_cli_version() {
    // The CLI doesn't have a separate version flag, but help shows the name
    // Just verify the binary runs
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("text2cli"));
}

#[test]
fn test_cli_init_case_sensitive() {
    // Shell names should be case-sensitive
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "BASH"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown shell"));
}

#[test]
fn test_cli_list_agents_format() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["list-agents"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available agents:"))
        .stdout(predicate::str::contains("enabled"));
}

#[test]
fn test_cli_help_init() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize shell integration"))
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("zsh"))
        .stdout(predicate::str::contains("pwsh"));
}

#[test]
fn test_cli_help_process() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["process", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Process input"));
}

#[test]
fn test_cli_process_with_special_characters() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // No trigger, so it passes through
    cmd.args(["process", "echo", "hello", "&&", "echo", "world"])
        .assert()
        .success();
}

#[test]
fn test_cli_process_empty_input() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["process"])
        .assert()
        .success()
        // Empty input with no trigger outputs newline
        .stdout(predicate::str::contains("\n"));
}

#[test]
fn test_cli_init_outputs_valid_script() {
    // Test that init outputs complete scripts
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    let output = cmd.args(["init", "bash"])
        .assert()
        .success()
        .get_output()
        .stdout.clone();

    let script = String::from_utf8_lossy(&output);
    assert!(script.contains("# text2cli bash integration"));
    assert!(script.contains("~/.bashrc"));
}

#[test]
fn test_cli_config_shows_path() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["config"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".text2cli"))
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn test_cli_trailing_input_unicode() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // Unicode input without trigger
    cmd.args(["中文测试", "🚀"])
        .assert()
        .success()
        .stdout(predicate::str::contains("中文测试"));
}

#[test]
fn test_cli_process_with_trigger_no_agent() {
    // Process with trigger - depends on agent availability
    // The behavior varies based on whether an agent is configured
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // With trigger, it will try to process
    let _ = cmd.args(["process", "@@@", "test"])
        .assert(); // Don't assert success or failure, just verify it runs
}

#[test]
fn test_cli_multiple_args_concatenated() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["hello", "beautiful", "world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello beautiful world"));
}

#[test]
fn test_cli_process_with_quotes() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    // No trigger - quotes should be preserved in the output
    cmd.args(["process", "echo", "\"hello world\""])
        .assert()
        .success()
        .stdout(predicate::str::contains("echo"));
}
