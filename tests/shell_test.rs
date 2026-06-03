use text2cli::{PwshHook, BashHook, ZshHook, ShellHook};

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

    // Should contain the custom binary name in variable assignment
    assert!(script.contains("${global:__Text2Cli_Binary__} = \"my-custom-cli\""));
}

// ============================================================================
// Bash Hook Tests
// ============================================================================

#[test]
fn test_bash_hook_generate() {
    let hook = BashHook::new("text2cli");
    let script = hook.generate();

    assert!(script.contains("text2cli"));
    assert!(script.contains("__text2cli_accept_line__"));
    assert!(script.contains("__text2cli_debug_trap__"));
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

    // Should contain the custom binary name in variable assignment
    assert!(script.contains("__TEXT2CLI_BINARY__=\"my-tool\""));
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

    // Should contain the custom binary name in variable assignment
    assert!(script.contains("__TEXT2CLI_BINARY__=\"my-cli-tool\""));
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

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_bash_hook_contains_trigger() {
    let hook = BashHook::new("text2cli");
    let generated = hook.generate();

    assert!(generated.contains("@@@"));
    assert!(generated.contains("__text2cli_accept_line__"));
}

#[test]
fn test_zsh_hook_contains_trigger() {
    let hook = ZshHook::new("text2cli");
    let generated = hook.generate();

    assert!(generated.contains("@@@"));
    assert!(generated.contains("__text2cli_preexec__"));
    assert!(generated.contains("print -z"));
}

#[test]
fn test_pwsh_hook_contains_trigger() {
    let hook = PwshHook::new("text2cli");
    let generated = hook.generate();

    assert!(generated.contains("@@@"));
    assert!(generated.contains("PSConsoleReadLine"));
}

#[test]
fn test_bash_hook_valid_syntax() {
    let hook = BashHook::new("text2cli");
    let generated = hook.generate();

    // Check for valid bash constructs
    assert!(generated.contains("local "));
    assert!(generated.contains("if [["));
    assert!(generated.contains("fi"));
}

#[test]
fn test_zsh_hook_valid_syntax() {
    let hook = ZshHook::new("text2cli");
    let generated = hook.generate();

    // Check for valid zsh constructs
    assert!(generated.contains("local "));
    assert!(generated.contains("if [["));
    assert!(generated.contains("fi"));
    assert!(generated.contains("preexec_functions"));
}

#[test]
fn test_pwsh_hook_valid_syntax() {
    let hook = PwshHook::new("text2cli");
    let generated = hook.generate();

    // Check for valid PowerShell constructs
    assert!(generated.contains("function "));
    assert!(generated.contains("param("));
    assert!(generated.contains("Set-PSReadLineKeyHandler"));
}

#[test]
fn test_bash_hook_custom_binary_with_path() {
    let hook = BashHook::new("/usr/local/bin/text2cli");
    let generated = hook.generate();

    assert!(generated.contains("/usr/local/bin/text2cli"));
}

#[test]
fn test_zsh_hook_custom_binary_with_path() {
    let hook = ZshHook::new("/opt/text2cli/bin/text2cli");
    let generated = hook.generate();

    assert!(generated.contains("/opt/text2cli/bin/text2cli"));
}

#[test]
fn test_pwsh_hook_custom_binary_with_path() {
    let hook = PwshHook::new("C:\\Tools\\text2cli.exe");
    let generated = hook.generate();

    assert!(generated.contains("C:\\Tools\\text2cli.exe"));
}

#[test]
fn test_should_intercept_various_inputs() {
    let hook = BashHook::new("text2cli");

    // Should intercept
    assert!(hook.should_intercept("@@@ command", "@@@"));
    assert!(hook.should_intercept("  @@@ command", "@@@"));
    assert!(hook.should_intercept("@@@   command", "@@@"));

    // Should not intercept
    assert!(!hook.should_intercept("echo @@@", "@@@"));
    assert!(!hook.should_intercept("no trigger", "@@@"));
    assert!(!hook.should_intercept("", "@@@"));
}

#[test]
fn test_should_intercept_suffix_trigger() {
    // By default, should_intercept checks for prefix
    let hook = BashHook::new("text2cli");

    // This tests the trait implementation
    assert!(hook.should_intercept("@@@ command", "@@@"));
    assert!(!hook.should_intercept("command @@@", "@@@"));
}

#[test]
fn test_all_hooks_same_interface() {
    // All hooks should implement the same trait
    let hooks: Vec<Box<dyn ShellHook>> = vec![
        Box::new(BashHook::new("text2cli")),
        Box::new(ZshHook::new("text2cli")),
        Box::new(PwshHook::new("text2cli")),
    ];

    for hook in hooks {
        // All should have a name
        assert!(!hook.name().is_empty());

        // All should generate some script
        let script = hook.generate();
        assert!(!script.is_empty());

        // All should have the trigger
        assert!(script.contains("@@@"));
    }
}

#[test]
fn test_hook_generate_is_idempotent() {
    let hook = BashHook::new("text2cli");

    let first = hook.generate();
    let second = hook.generate();

    assert_eq!(first, second);
}

#[test]
fn test_bash_hook_readline_variable() {
    let hook = BashHook::new("text2cli");
    let generated = hook.generate();

    // Bash uses READLINE_LINE for command injection
    assert!(generated.contains("READLINE_LINE"));
}

#[test]
fn test_zsh_hook_print_z() {
    let hook = ZshHook::new("text2cli");
    let generated = hook.generate();

    // Zsh uses print -z for command injection
    assert!(generated.contains("print -z"));
}

#[test]
fn test_pwsh_hook_replace() {
    let hook = PwshHook::new("text2cli");
    let generated = hook.generate();

    // PowerShell uses Replace for command injection
    assert!(generated.contains("Replace"));
}

#[test]
fn test_hook_with_unicode_binary_name() {
    let hook = BashHook::new("text2cli-🚀");
    let generated = hook.generate();

    assert!(generated.contains("text2cli-🚀"));
}

#[test]
fn test_hook_with_spaces_in_binary_name() {
    let hook = BashHook::new("my text2cli");
    let generated = hook.generate();

    assert!(generated.contains("my text2cli"));
}
