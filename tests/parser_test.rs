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
