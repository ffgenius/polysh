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
    // Verify Dialect::from_str works for all known aliases
    assert_eq!(Dialect::from_str("powershell"), Some(Dialect::PowerShell));
    assert_eq!(Dialect::from_str("ps"), Some(Dialect::PowerShell));
    assert_eq!(Dialect::from_str("bash"), Some(Dialect::Unix));
    assert_eq!(Dialect::from_str("zsh"), Some(Dialect::Unix));
    assert_eq!(Dialect::from_str("cmd"), Some(Dialect::Cmd));
    assert_eq!(Dialect::from_str("unknown"), None);
}
