use text2cli::parser::{ParsePosition, TriggerParser};

#[test]
fn test_parse_prefix_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@ 重命名这个变量").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
    assert_eq!(cmd.position, ParsePosition::Prefix);
}

#[test]
fn test_parse_suffix_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("重命名这个变量 @@@").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
    assert_eq!(cmd.position, ParsePosition::Suffix);
}

#[test]
fn test_no_trigger() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("echo hello").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_empty_content() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_whitespace_handling() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("@@@   重命名这个变量   ").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");
}

#[test]
fn test_custom_trigger() {
    let parser = TriggerParser::new("!!!");

    let result = parser.parse("!!! fix this bug").unwrap();
    assert!(result.is_some());

    let cmd = result.unwrap();
    assert_eq!(cmd.content, "fix this bug");
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_trigger_only_whitespace_content() {
    let parser = TriggerParser::new("@@@");

    // Trigger followed by only whitespace should return None
    let result = parser.parse("@@@    ").unwrap();
    assert!(result.is_none(), "Expected None for trigger with only whitespace");

    let result = parser.parse("@@@\t\n").unwrap();
    assert!(result.is_none(), "Expected None for trigger with tabs/newlines");
}

#[test]
fn test_multiline_input() {
    let parser = TriggerParser::new("@@@");

    // Multi-line command
    let result = parser.parse("@@@ first line\nsecond line").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert!(cmd.content.contains("first line"));
    assert!(cmd.content.contains("second line"));
}

#[test]
fn test_special_characters_in_content() {
    let parser = TriggerParser::new("@@@");

    // Content with special characters
    let result = parser.parse("@@@ git commit -m \"fix: issue #123\"").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "git commit -m \"fix: issue #123\"");

    // Pipes and redirects
    let result = parser.parse("@@@ cat file | grep pattern > output").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "cat file | grep pattern > output");
}

#[test]
fn test_trigger_at_both_ends() {
    let parser = TriggerParser::new("@@@");

    // Prefix takes precedence when trigger appears at both ends
    let result = parser.parse("@@@ content @@@").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.position, ParsePosition::Prefix);
    assert_eq!(cmd.content, "content @@@");
}

#[test]
fn test_very_long_input() {
    let parser = TriggerParser::new("@@@");

    let long_content = "x".repeat(10000);
    let input = format!("@@@ {}", long_content);

    let result = parser.parse(&input).unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content.len(), 10000);
}

#[test]
fn test_unicode_content() {
    let parser = TriggerParser::new("@@@");

    // Chinese characters
    let result = parser.parse("@@@ 重命名这个变量").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "重命名这个变量");

    // Emoji
    let result = parser.parse("@@@ 🚀 deploy to production").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "🚀 deploy to production");

    // Mixed unicode
    let result = parser.parse("@@@ 中文 日本語 한글 العربية").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "中文 日本語 한글 العربية");
}

#[test]
fn test_different_trigger_characters() {
    // Test various trigger patterns
    let parsers_and_inputs = vec![
        ("!!!", "!!! command", "command"),
        ("$$$", "$$$ test", "test"),
        ("###", "### run tests", "run tests"),
        (">>>", ">>> execute", "execute"),
        ("[[[", "[[[ brackets", "brackets"),
    ];

    for (trigger, input, expected) in parsers_and_inputs {
        let parser = TriggerParser::new(trigger);
        let result = parser.parse(input).unwrap();
        assert!(result.is_some(), "Failed for trigger: {}", trigger);
        assert_eq!(result.unwrap().content, expected);
    }
}

#[test]
fn test_empty_input() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("").unwrap();
    assert!(result.is_none(), "Expected None for empty input");

    let result = parser.parse("   ").unwrap();
    assert!(result.is_none(), "Expected None for whitespace-only input");
}

#[test]
fn test_raw_input_preserved() {
    let parser = TriggerParser::new("@@@");

    let input = "  @@@ hello world  ";
    let result = parser.parse(input).unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.raw_input, input);
}

#[test]
fn test_suffix_trigger_trims_correctly() {
    let parser = TriggerParser::new("@@@");

    let result = parser.parse("hello world   @@@").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.position, ParsePosition::Suffix);
    assert_eq!(cmd.content, "hello world");
}

#[test]
fn test_single_character_trigger() {
    let parser = TriggerParser::new(">");

    let result = parser.parse("> run this").unwrap();
    assert!(result.is_some());
    let cmd = result.unwrap();
    assert_eq!(cmd.content, "run this");
}

#[test]
fn test_trigger_case_sensitivity() {
    // Trigger should be case-sensitive
    let parser = TriggerParser::new("ABC");

    let result = parser.parse("ABC command").unwrap();
    assert!(result.is_some());

    let result = parser.parse("abc command").unwrap();
    assert!(result.is_none(), "Expected None for lowercase trigger");
}

#[test]
fn test_parse_position_equality() {
    assert_eq!(ParsePosition::Prefix, ParsePosition::Prefix);
    assert_eq!(ParsePosition::Suffix, ParsePosition::Suffix);
    assert_ne!(ParsePosition::Prefix, ParsePosition::Suffix);
}

#[test]
fn test_parsed_command_debug() {
    let parser = TriggerParser::new("@@@");
    let result = parser.parse("@@@ test").unwrap().unwrap();

    let debug = format!("{:?}", result);
    assert!(debug.contains("ParsedCommand"));
    assert!(debug.contains("test"));
    assert!(debug.contains("Prefix"));
}
