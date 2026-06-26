//! Tests migrated from src/detector.rs — shell detection.

use polysh::detector::detect_shell;
use polysh::mappings::Dialect;

#[test]
fn test_detect_shell_does_not_panic() {
    // Just verify detection runs without panicking
    let info = detect_shell();
    // Should always return something valid
    assert!(info.version.is_none() || info.version.unwrap() >= 1);
}

#[test]
fn test_dialect_from_str_aliases() {
    // Verify Dialect::parse works for all known aliases
    assert_eq!(Dialect::parse("powershell"), Some(Dialect::PowerShell));
    assert_eq!(Dialect::parse("ps"), Some(Dialect::PowerShell));
    assert_eq!(Dialect::parse("bash"), Some(Dialect::Unix));
    assert_eq!(Dialect::parse("zsh"), Some(Dialect::Unix));
    assert_eq!(Dialect::parse("cmd"), Some(Dialect::Cmd));
    assert_eq!(Dialect::parse("unknown"), None);
}
