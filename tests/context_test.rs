use serial_test::serial;
use std::process::Command;
use tempfile::tempdir;
use text2cli::{Context, ContextCollector, GitStatus};

// These tests change the current directory, so they must run serially
#[test]
#[serial]
fn test_collect_working_dir() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let expected_suffix = dir.path().file_name().unwrap();
    assert!(
        context.working_dir.ends_with(expected_suffix),
        "working_dir {:?} should end with {:?}",
        context.working_dir,
        expected_suffix
    );
}

#[test]
#[serial]
fn test_context_has_git_status_in_non_git_dir() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    // Not a git repo, so git_status should be None
    let context = result.expect("ContextCollector::collect should not fail in non-git dir");
    assert!(context.git_status.is_none());
}

#[test]
fn test_context_default() {
    let context = Context::default();
    assert!(context.git_branch.is_none());
    assert!(context.git_status.is_none());
    assert!(context.recent_files.is_empty());
}

// =============================================================================
// GitStatus parsing tests
// =============================================================================

/// Helper to run git commands in a directory
fn git_cmd(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git command should succeed");
}

/// Helper to write a file in a directory
fn write_file(dir: &std::path::Path, name: &str, content: &str) {
    std::fs::write(dir.join(name), content).expect("write should succeed");
}

#[test]
#[serial]
fn test_git_status_modified_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create and commit a file
    write_file(dir.path(), "modified.txt", "original content");
    git_cmd(dir.path(), &["add", "modified.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    // Modify the file (unstaged)
    write_file(dir.path(), "modified.txt", "modified content");

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    assert_eq!(status.modified, 1, "Should have 1 modified file");
    assert_eq!(status.added, 0);
    assert_eq!(status.deleted, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
#[serial]
fn test_git_status_added_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create and stage a new file
    write_file(dir.path(), "added.txt", "new file");
    git_cmd(dir.path(), &["add", "added.txt"]);

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    assert_eq!(status.added, 1, "Should have 1 added file");
    assert_eq!(status.modified, 0);
    assert_eq!(status.deleted, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
#[serial]
fn test_git_status_deleted_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create, commit, then delete a file
    write_file(dir.path(), "deleted.txt", "to be deleted");
    git_cmd(dir.path(), &["add", "deleted.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);
    std::fs::remove_file(dir.path().join("deleted.txt")).unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    assert_eq!(status.deleted, 1, "Should have 1 deleted file");
    assert_eq!(status.modified, 0);
    assert_eq!(status.added, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
#[serial]
fn test_git_status_untracked_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create an untracked file (no git add)
    write_file(dir.path(), "untracked.txt", "untracked content");

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    assert_eq!(status.untracked, 1, "Should have 1 untracked file");
    assert_eq!(status.modified, 0);
    assert_eq!(status.added, 0);
    assert_eq!(status.deleted, 0);
}

#[test]
#[serial]
fn test_git_status_combined_mm() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create, commit, stage modification, then modify again (MM status)
    write_file(dir.path(), "mm.txt", "original");
    git_cmd(dir.path(), &["add", "mm.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);
    write_file(dir.path(), "mm.txt", "staged modification");
    git_cmd(dir.path(), &["add", "mm.txt"]);
    write_file(dir.path(), "mm.txt", "unstaged modification");

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    // MM is counted as modified
    assert_eq!(status.modified, 1, "MM status should count as modified");
}

#[test]
#[serial]
fn test_git_status_combined_am() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create, stage, then modify again (AM status)
    write_file(dir.path(), "am.txt", "new file");
    git_cmd(dir.path(), &["add", "am.txt"]);
    write_file(dir.path(), "am.txt", "modified after staging");

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    // AM is counted as added
    assert_eq!(status.added, 1, "AM status should count as added");
}

#[test]
#[serial]
fn test_git_status_multiple_files() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create initial commit
    write_file(dir.path(), "file1.txt", "content1");
    git_cmd(dir.path(), &["add", "file1.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    // Create and commit file to be deleted later
    write_file(dir.path(), "to_delete.txt", "will be deleted");
    git_cmd(dir.path(), &["add", "to_delete.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "add file to delete"]);

    // Now create multiple file states:
    // 1. Modified file (unstaged)
    write_file(dir.path(), "file1.txt", "modified content");
    // 2. Untracked file
    write_file(dir.path(), "untracked.txt", "new file");
    // 3. Staged for addition (added)
    write_file(dir.path(), "added.txt", "staged file");
    git_cmd(dir.path(), &["add", "added.txt"]);
    // 4. Deleted file (unstaged deletion)
    std::fs::remove_file(dir.path().join("to_delete.txt")).unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    assert_eq!(status.modified, 1, "Should have 1 modified");
    assert_eq!(status.untracked, 1, "Should have 1 untracked");
    assert_eq!(status.added, 1, "Should have 1 added");
    assert_eq!(status.deleted, 1, "Should have 1 deleted");
}

// =============================================================================
// git_branch tests
// =============================================================================

#[test]
#[serial]
fn test_git_branch_in_git_repo() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create initial commit (required for branch to exist)
    write_file(dir.path(), "README.md", "# test");
    git_cmd(dir.path(), &["add", "README.md"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    // Default branch name varies (main, master), just check it exists
    assert!(
        context.git_branch.is_some(),
        "git_branch should be Some in git repo with commits"
    );
}

#[test]
#[serial]
fn test_git_branch_in_project_directory() {
    // This test runs in the actual project directory which should be a git repo
    let original_dir = std::env::current_dir().unwrap();

    // The project directory is a git repo
    let result = ContextCollector::collect();

    // Restore directory
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    assert!(
        context.git_branch.is_some(),
        "Project directory should have a git branch"
    );
}

// =============================================================================
// get_recent_files tests (stub behavior)
// =============================================================================

#[test]
#[serial]
fn test_get_recent_files_returns_empty() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    // get_recent_files is currently a stub that returns empty Vec
    assert!(
        context.recent_files.is_empty(),
        "get_recent_files should return empty Vec (stub implementation)"
    );
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
#[serial]
fn test_git_status_renamed_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create, commit, then rename a file
    write_file(dir.path(), "old_name.txt", "content");
    git_cmd(dir.path(), &["add", "old_name.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    // Rename the file
    std::fs::rename(dir.path().join("old_name.txt"), dir.path().join("new_name.txt")).unwrap();
    git_cmd(dir.path(), &["add", "old_name.txt", "new_name.txt"]);

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some in git repo");
    // Renamed files are counted as modified or added depending on git status output
    // Just verify we got a valid status
    let _ = status;
}

#[test]
#[serial]
fn test_git_status_staged_and_unstaged_changes() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create and commit a file
    write_file(dir.path(), "file1.txt", "original");
    git_cmd(dir.path(), &["add", "file1.txt"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    // Stage one change
    write_file(dir.path(), "file1.txt", "staged change");
    git_cmd(dir.path(), &["add", "file1.txt"]);

    // Make another unstaged change
    write_file(dir.path(), "file1.txt", "unstaged change");

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    let status = context.git_status.expect("git_status should be Some");
    // Should have at least one modified file
    assert!(status.modified >= 1 || status.added >= 1);
}

#[test]
#[serial]
fn test_git_status_empty_repo() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize empty git repo (no commits)
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    // Empty repo still has git status (all zeros)
    assert!(context.git_status.is_some());
    let status = context.git_status.unwrap();
    assert_eq!(status.modified, 0);
    assert_eq!(status.added, 0);
    assert_eq!(status.deleted, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
#[serial]
fn test_git_branch_detached_head() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Initialize git repo
    git_cmd(dir.path(), &["init"]);
    git_cmd(dir.path(), &["config", "user.email", "test@test.com"]);
    git_cmd(dir.path(), &["config", "user.name", "Test"]);

    // Create initial commit
    write_file(dir.path(), "README.md", "# test");
    git_cmd(dir.path(), &["add", "README.md"]);
    git_cmd(dir.path(), &["commit", "-m", "initial"]);

    // Get commit hash and checkout detached
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .expect("git rev-parse should succeed");
    let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Checkout detached HEAD
    git_cmd(dir.path(), &["checkout", &commit_hash]);

    std::env::set_current_dir(dir.path()).unwrap();
    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    // Detached HEAD - branch might be None or empty
    // git branch --show-current returns empty for detached HEAD
    assert!(context.git_branch.is_none() || context.git_branch.as_ref().unwrap().is_empty());
}

#[test]
fn test_context_debug() {
    let context = Context::default();
    let debug = format!("{:?}", context);

    assert!(debug.contains("Context"));
    assert!(debug.contains("working_dir"));
    assert!(debug.contains("git_branch"));
}

#[test]
fn test_git_status_debug() {
    let status = GitStatus {
        modified: 1,
        added: 2,
        deleted: 3,
        untracked: 4,
    };

    let debug = format!("{:?}", status);
    assert!(debug.contains("modified: 1"));
    assert!(debug.contains("added: 2"));
    assert!(debug.contains("deleted: 3"));
    assert!(debug.contains("untracked: 4"));
}

#[test]
fn test_git_status_default() {
    let status = GitStatus::default();

    assert_eq!(status.modified, 0);
    assert_eq!(status.added, 0);
    assert_eq!(status.deleted, 0);
    assert_eq!(status.untracked, 0);
}

#[test]
fn test_git_status_clone() {
    let status = GitStatus {
        modified: 5,
        added: 3,
        deleted: 1,
        untracked: 10,
    };

    let cloned = status.clone();
    assert_eq!(cloned.modified, 5);
    assert_eq!(cloned.added, 3);
    assert_eq!(cloned.deleted, 1);
    assert_eq!(cloned.untracked, 10);
}

#[test]
fn test_context_clone() {
    let mut context = Context::default();
    context.working_dir = std::path::PathBuf::from("/test/path");
    context.git_branch = Some("feature".to_string());

    let cloned = context.clone();
    assert_eq!(cloned.working_dir, std::path::PathBuf::from("/test/path"));
    assert_eq!(cloned.git_branch, Some("feature".to_string()));
}

#[test]
#[serial]
fn test_context_with_shell_env() {
    let original_dir = std::env::current_dir().unwrap();

    // Set some test environment variables
    std::env::set_var("SHELL_TEST_CTX", "shell_value");
    std::env::set_var("TERM_TEST_CTX", "term_value");

    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");

    // Shell env should contain our test vars
    assert!(context.shell_env.contains_key("SHELL_TEST_CTX"));
    assert!(context.shell_env.contains_key("TERM_TEST_CTX"));

    // Cleanup
    std::env::remove_var("SHELL_TEST_CTX");
    std::env::remove_var("TERM_TEST_CTX");
}

#[test]
#[serial]
fn test_working_dir_with_special_characters() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("path with spaces");
    std::fs::create_dir_all(&subdir).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&subdir).unwrap();

    let result = ContextCollector::collect();
    std::env::set_current_dir(&original_dir).unwrap();

    let context = result.expect("ContextCollector::collect should not fail");
    assert!(context.working_dir.to_string_lossy().contains("path with spaces"));
}
