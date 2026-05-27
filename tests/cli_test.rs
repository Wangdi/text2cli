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
fn test_cli_init_bash() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("preexec"));
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
fn test_cli_list_agents() {
    let mut cmd = Command::cargo_bin("text2cli").unwrap();
    cmd.args(["list-agents"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-code"));
}
