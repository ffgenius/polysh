//! Shell detection based on platform heuristics.
//!
//! Detects whether the current platform is running PowerShell, CMD, or a Unix-style
//! shell, and reports its capabilities.

use std::env;

use crate::mappings::Dialect;

/// Information about the current shell environment.
#[derive(Debug, Clone)]
pub struct ShellInfo {
    /// The type of shell currently running.
    pub dialect: Dialect,
    /// Whether this shell natively supports `&&` and `||` conditional connectors.
    pub supports_conditional_connectors: bool,
    /// Whether this shell needs Unix command translation (e.g., PowerShell/CMD do).
    pub needs_unix_translation: bool,
    /// The target dialect for command translation within this shell.
    pub target: Dialect,
    /// PowerShell major version, if available.
    pub version: Option<u32>,
}

impl Default for ShellInfo {
    fn default() -> Self {
        Self {
            dialect: Dialect::Unix,
            supports_conditional_connectors: true,
            needs_unix_translation: false,
            target: Dialect::Unix,
            version: None,
        }
    }
}

/// Detect the current shell environment based on platform heuristics.
pub fn detect_shell() -> ShellInfo {
    if cfg!(target_os = "windows") {
        detect_windows_shell()
    } else {
        detect_unix_shell()
    }
}

#[cfg(target_os = "windows")]
fn shell_info_for(dialect: Dialect) -> ShellInfo {
    match dialect {
        Dialect::PowerShell => ShellInfo {
            dialect: Dialect::PowerShell,
            // Assume PowerShell 7+ (which supports && / ||). If the user is on
            // an older version, they can construct ShellInfo manually.
            supports_conditional_connectors: true,
            needs_unix_translation: true,
            target: Dialect::PowerShell,
            version: None,
        },
        Dialect::Cmd => ShellInfo {
            dialect: Dialect::Cmd,
            supports_conditional_connectors: true,
            needs_unix_translation: true,
            target: Dialect::Cmd,
            version: None,
        },
        _ => ShellInfo {
            dialect,
            supports_conditional_connectors: true,
            needs_unix_translation: false,
            target: dialect,
            version: None,
        },
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_shell() -> ShellInfo {
    // CMD: PROMPT is set but PSModulePath is not
    let is_cmd = env::var("PROMPT").is_ok() && env::var("PSModulePath").is_err();
    if is_cmd {
        return shell_info_for(Dialect::Cmd);
    }

    // PowerShell: PSModulePath is set
    if env::var("PSModulePath").is_ok() {
        return shell_info_for(Dialect::PowerShell);
    }

    // ComSpec containing cmd.exe
    if let Ok(comspec) = env::var("ComSpec") {
        if comspec.to_lowercase().contains("cmd.exe") {
            return shell_info_for(Dialect::Cmd);
        }
    }

    // Git Bash / WSL bash via SHELL
    if let Ok(shell_path) = env::var("SHELL") {
        if shell_path.contains("bash") {
            return ShellInfo {
                dialect: Dialect::Unix,
                supports_conditional_connectors: true,
                needs_unix_translation: true,
                target: Dialect::Unix,
                version: None,
            };
        }
    }

    // Fallback to PowerShell
    shell_info_for(Dialect::PowerShell)
}

#[cfg(not(target_os = "windows"))]
fn detect_windows_shell() -> ShellInfo {
    ShellInfo::default()
}

#[cfg(not(target_os = "windows"))]
fn detect_unix_shell() -> ShellInfo {
    if let Ok(shell_path) = env::var("SHELL") {
        let name = shell_path
            .rsplit('/')
            .next()
            .unwrap_or(&shell_path)
            .to_lowercase();

        // All common Unix shells have the same capabilities for our purposes
        let is_known = name.contains("bash")
            || name.contains("zsh")
            || name.contains("fish")
            || name.contains("dash")
            || name.contains("ash")
            || name.contains("ksh")
            || name.contains("tcsh")
            || name.contains("busybox");

        if is_known {
            return ShellInfo::default();
        }
    }

    ShellInfo::default()
}

#[cfg(target_os = "windows")]
fn detect_unix_shell() -> ShellInfo {
    // On Windows without SHELL, default to PowerShell
    shell_info_for(Dialect::PowerShell)
}
