use serial_test::serial;
use tempfile::tempdir;
use text2cli::context::{Context, ContextCollector};

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
