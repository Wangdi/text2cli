use text2cli::shell::{PwshHook, BashHook, ZshHook, ShellHook};

// ============================================================================
// PowerShell Hook Tests
// ============================================================================

#[test]
fn test_pwsh_hook_generate() {
    let hook = PwshHook::new("text2cli");
    let script = hook.generate();

    assert!(script.contains("text2cli"));
    assert!(script.contains("PSReadLine"));
    assert!(script.contains("Set-PSReadLineKeyHandler"));
    assert!(script.contains("GetBufferState"));
}

#[test]
fn test_pwsh_hook_name() {
    let hook = PwshHook::new("text2cli");
    assert_eq!(hook.name(), "pwsh");
}

#[test]
fn test_pwsh_hook_trigger_detection() {
    let hook = PwshHook::new("text2cli");
    let trigger = "@@@";

    // Should intercept
    assert!(hook.should_intercept("@@@ test", trigger));
    assert!(hook.should_intercept("@@@list files", trigger));
    assert!(hook.should_intercept("  @@@  trimmed", trigger));

    // Should not intercept
    assert!(!hook.should_intercept("echo hello", trigger));
    assert!(!hook.should_intercept("echo @@@ test", trigger));
    assert!(!hook.should_intercept("", trigger));
}

#[test]
fn test_pwsh_hook_custom_binary() {
    let hook = PwshHook::new("my-custom-cli");
    let script = hook.generate();

    // Should contain the custom binary name in command calls
    assert!(script.contains("my-custom-cli process"));
}

// ============================================================================
// Bash Hook Tests
// ============================================================================

#[test]
fn test_bash_hook_generate() {
    let hook = BashHook::new("text2cli");
    let script = hook.generate();

    assert!(script.contains("text2cli"));
    assert!(script.contains("preexec"));
    assert!(script.contains("__text2cli_preexec__"));
    assert!(script.contains("READLINE_LINE"));
}

#[test]
fn test_bash_hook_name() {
    let hook = BashHook::new("text2cli");
    assert_eq!(hook.name(), "bash");
}

#[test]
fn test_bash_hook_trigger_detection() {
    let hook = BashHook::new("text2cli");
    let trigger = "@@@";

    // Should intercept
    assert!(hook.should_intercept("@@@ test", trigger));
    assert!(hook.should_intercept("@@@git status", trigger));

    // Should not intercept
    assert!(!hook.should_intercept("echo @@@", trigger));
    assert!(!hook.should_intercept("ls -la", trigger));
}

#[test]
fn test_bash_hook_custom_binary() {
    let hook = BashHook::new("my-tool");
    let script = hook.generate();

    // Should contain the custom binary name in command calls
    assert!(script.contains("my-tool process"));
}

// ============================================================================
// Zsh Hook Tests
// ============================================================================

#[test]
fn test_zsh_hook_generate() {
    let hook = ZshHook::new("text2cli");
    let script = hook.generate();

    assert!(script.contains("text2cli"));
    assert!(script.contains("print -z"));
    assert!(script.contains("preexec_functions"));
    assert!(script.contains("__text2cli_preexec__"));
}

#[test]
fn test_zsh_hook_name() {
    let hook = ZshHook::new("text2cli");
    assert_eq!(hook.name(), "zsh");
}

#[test]
fn test_zsh_hook_trigger_detection() {
    let hook = ZshHook::new("text2cli");
    let trigger = "@@@";

    // Should intercept
    assert!(hook.should_intercept("@@@ test", trigger));
    assert!(hook.should_intercept("  @@@   spaced", trigger));

    // Should not intercept
    assert!(!hook.should_intercept("echo @@@ test", trigger));
    assert!(!hook.should_intercept("pwd", trigger));
}

#[test]
fn test_zsh_hook_custom_binary() {
    let hook = ZshHook::new("my-cli-tool");
    let script = hook.generate();

    // Should contain the custom binary name in command calls
    assert!(script.contains("my-cli-tool process"));
}

// ============================================================================
// Generic ShellHook Trait Tests
// ============================================================================

#[test]
fn test_shell_hook_trait_object() {
    // Test that we can use hooks as trait objects
    let hooks: Vec<Box<dyn ShellHook>> = vec![
        Box::new(PwshHook::new("text2cli")),
        Box::new(BashHook::new("text2cli")),
        Box::new(ZshHook::new("text2cli")),
    ];

    let names: Vec<&str> = hooks.iter().map(|h| h.name()).collect();
    assert_eq!(names, vec!["pwsh", "bash", "zsh"]);

    // All should intercept trigger
    for hook in &hooks {
        assert!(hook.should_intercept("@@@ test", "@@@"));
    }
}

#[test]
fn test_custom_trigger() {
    let hook = PwshHook::new("text2cli");

    // Custom trigger "!!!"
    assert!(hook.should_intercept("!!! do something", "!!!"));
    assert!(!hook.should_intercept("@@@ test", "!!!"));
}
