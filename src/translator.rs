//! Bidirectional shell command translation engine.
//!
//! The translator is **direction-agnostic**: it takes a source dialect and a target
//! dialect, tokenizes the input, splits by connectors (`&&`, `||`) and pipes (`|`),
//! translates each segment via the `MappingRegistry`, and reassembles the result.

use crate::detector::ShellInfo;
use crate::mappings::dynamic::translate_dynamic;
use crate::mappings::{Dialect, MappingRegistry};
use crate::tokenizer::{tokenize_with_pos_enhanced_and_roles, RoleToken, TokenRole};

/// Result of linting a command for unsupported segments.
#[derive(Debug, Clone)]
pub struct LintResult {
    pub unsupported: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Detect the input format (dialect) of a command string by keyword heuristics.
pub fn detect_input_format(command: &str) -> Dialect {
    // PowerShell indicators
    let ps_indicators = [
        "Remove-Item",
        "Get-ChildItem",
        "Copy-Item",
        "Move-Item",
        "New-Item",
        "Get-Content",
        "Select-String",
        "Write-Host",
        "Clear-Host",
        "Get-Location",
        "$env:",
        "Invoke-",
    ];
    for indicator in &ps_indicators {
        if command.contains(indicator) {
            return Dialect::PowerShell;
        }
    }

    // Unix command indicators
    let unix_indicators = [
        "grep", "ls ", "cat ", "rm ", "cp ", "mv ", "mkdir", "chmod", "chown", "sed ", "awk ",
    ];
    for indicator in &unix_indicators {
        if command.contains(indicator) {
            return Dialect::Unix;
        }
    }
    // Word-boundary Unix check
    if regex_boundary_match(command, "find")
        || regex_boundary_match(command, "ls")
        || regex_boundary_match(command, "rm")
    {
        return Dialect::Unix;
    }

    // CMD indicators
    let cmd_indicators = [
        "del ", "dir ", "copy ", "move ", "md ", "type ", "findstr", "cls", "tasklist", "taskkill",
    ];
    for indicator in &cmd_indicators {
        if command.to_lowercase().contains(indicator) {
            return Dialect::Cmd;
        }
    }
    if command.contains("echo %") {
        return Dialect::Cmd;
    }

    // Default to Unix
    Dialect::Unix
}

/// Simple word-boundary regex check.
fn regex_boundary_match(text: &str, word: &str) -> bool {
    let lower = text.to_lowercase();
    // Check for word boundaries using simple string matching
    if lower.starts_with(word)
        && (lower.len() == word.len() || lower.as_bytes()[word.len()].is_ascii_whitespace())
    {
        return true;
    }
    let pattern = format!(" {} ", word);
    if lower.contains(&pattern) {
        return true;
    }
    false
}

/// Translate a shell command from its detected source dialect to the target dialect.
///
/// This is the main entry point for translation. It:
/// 1. Detects the input dialect
/// 2. Splits the command by `&&`/`||` connectors (respecting nesting)
/// 3. Splits each segment by `|` pipes (respecting nesting)
/// 4. Translates each pipe segment
/// 5. Handles conditional connectors for shells that don't natively support them
/// 6. Reassembles the translated command
pub fn translate_command(command: &str, shell: &ShellInfo) -> String {
    let registry = MappingRegistry::new();
    let input_format = detect_input_format(command);

    translate_with_registry(command, input_format, shell.target, shell, &registry)
}

/// Translate a command with an explicit source and target, using a pre-built registry.
pub fn translate_with_registry(
    command: &str,
    source: Dialect,
    target: Dialect,
    shell: &ShellInfo,
    registry: &MappingRegistry,
) -> String {
    // If source == target, no translation needed
    if source == target && !shell.needs_unix_translation {
        return command.to_string();
    }

    // Handle PowerShell backtick-escaped operators first
    let command = handle_backtick_escapes(command);

    // Split by && and || connectors
    let parts = split_by_connectors(&command);
    if parts.is_empty() {
        return command;
    }

    let mut translated_parts: Vec<String> = Vec::new();

    for part in &parts {
        match part.as_str() {
            "&&" | "||" => {
                translated_parts.push(part.clone());
            }
            _ => {
                // Split by pipes and translate each segment
                let pipe_parts = split_by_pipe(part);
                let translated_pipe: Vec<String> = pipe_parts
                    .iter()
                    .map(|seg| translate_segment(seg, source, target, registry))
                    .collect();
                translated_parts.push(translated_pipe.join(" | "));
            }
        }
    }

    let translated = translated_parts.join(" ");

    // Handle conditional connectors for shells that don't natively support them
    if shell.supports_conditional_connectors {
        return translated;
    }

    // Legacy PowerShell (< 7) needs special handling for && and ||
    if shell.dialect == Dialect::PowerShell {
        translate_for_legacy_powershell(&translated)
    } else {
        translated
    }
}

/// Translate a single command segment (no connectors, no top-level pipes).
fn translate_segment(
    segment: &str,
    source: Dialect,
    target: Dialect,
    registry: &MappingRegistry,
) -> String {
    let trimmed = segment.trim();

    // Skip environment variable expansions — pass through unchanged
    if trimmed.contains("${") {
        return trimmed.to_string();
    }

    // Skip subshells and groupings
    if trimmed.starts_with('(') || trimmed.starts_with('{') {
        return trimmed.to_string();
    }

    // Tokenize
    let tokens = tokenize_with_pos_enhanced_and_roles(trimmed);
    if tokens.is_empty() {
        return trimmed.to_string();
    }

    // Check for here-documents — pass through
    if has_here_doc(&tokens) {
        return trimmed.to_string();
    }

    // Find the command token
    let cmd_token = tokens.iter().find(|t| t.role == TokenRole::Cmd);
    if cmd_token.is_none() {
        return trimmed.to_string();
    }
    let cmd_token = cmd_token.unwrap();

    // Extract flags and args
    let flags: Vec<String> = tokens
        .iter()
        .filter(|t| t.role == TokenRole::Flag)
        .map(|t| t.reconstructed_value.clone())
        .collect();

    let args: Vec<String> = tokens
        .iter()
        .filter(|t| t.role == TokenRole::Arg)
        .map(|t| t.reconstructed_value.clone())
        .collect();

    // Reclassify CMD-style slash flags when source is CMD
    let flags = if source == Dialect::Cmd {
        reclassify_cmd_flags(&tokens, &flags)
    } else {
        flags
    };

    let cmd_name = &cmd_token.reconstructed_value;

    // Try dynamic translation first
    if let Some(result) = translate_dynamic(cmd_name, &flags, &args, source, target) {
        return result;
    }

    // Try static mapping
    if let Some(mapping) = registry.lookup_cmd(source, cmd_name) {
        let target_cmd = mapping.cmd_name(target);
        if target_cmd.is_empty() {
            // No translation available for this direction
            return trimmed.to_string();
        }

        // Translate flags
        let mut translated_flags = String::new();
        let mut unknown_flags = Vec::new();

        for flag in &flags {
            if let Some(tflag) = registry.translate_flag(source, cmd_name, flag, target) {
                if !tflag.is_empty() {
                    translated_flags.push(' ');
                    translated_flags.push_str(tflag);
                }
            } else {
                // Unknown flag — keep original
                unknown_flags.push(flag.clone());
            }
        }

        // If we have unknown flags and force_args is set, fail gracefully
        if !unknown_flags.is_empty() && mapping.force_args && args.is_empty() {
            return trimmed.to_string();
        }

        let mut result = target_cmd.to_string();
        result.push_str(&translated_flags);

        // Add remaining unknown flags
        for f in &unknown_flags {
            result.push(' ');
            result.push_str(f);
        }

        // Add arguments
        for arg in &args {
            result.push(' ');
            result.push_str(arg);
        }

        return result.trim().to_string();
    }

    // Unknown command — return as-is
    trimmed.to_string()
}

/// Check if tokens contain a here-document operator.
fn has_here_doc(tokens: &[RoleToken]) -> bool {
    let values: Vec<&str> = tokens
        .iter()
        .map(|t| t.reconstructed_value.as_str())
        .collect();
    for i in 0..values.len().saturating_sub(1) {
        if values[i] == "<" && values[i + 1] == "<" {
            return true;
        }
    }
    tokens.iter().any(|t| t.reconstructed_value == "<<")
}

/// Reclassify CMD-style flags (/s, /q) as flags instead of args.
fn reclassify_cmd_flags(tokens: &[RoleToken], flags: &[String]) -> Vec<String> {
    let mut result = flags.to_vec();
    for t in tokens {
        if t.role == TokenRole::Arg
            && t.quote_type.is_none()
            && t.reconstructed_value.starts_with('/')
            && t.reconstructed_value.len() > 1
            && t.reconstructed_value[1..]
                .chars()
                .all(|c| c.is_ascii_alphabetic())
            && !result.contains(&t.reconstructed_value)
        {
            result.push(t.reconstructed_value.clone());
        }
    }
    result
}

/// Split a command by `&&` and `||` connectors, respecting parentheses/brace depth.
fn split_by_connectors(cmd: &str) -> Vec<String> {
    let tokens = tokenize_with_pos_enhanced_and_roles(cmd);
    let mut parts: Vec<String> = Vec::new();
    let mut segment_start = 0usize;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;

    for t in &tokens {
        match t.reconstructed_value.as_str() {
            "(" => paren_depth += 1,
            ")" => paren_depth = 0.max(paren_depth - 1),
            "{" => brace_depth += 1,
            "}" => brace_depth = 0.max(brace_depth - 1),
            _ => {}
        }

        if paren_depth == 0
            && brace_depth == 0
            && (t.reconstructed_value == "&&" || t.reconstructed_value == "||")
        {
            let chunk = cmd[segment_start..t.start].trim();
            if !chunk.is_empty() {
                parts.push(chunk.to_string());
            }
            parts.push(t.reconstructed_value.clone());
            segment_start = t.end;
        }
    }

    let last = cmd[segment_start..].trim();
    if !last.is_empty() {
        parts.push(last.to_string());
    }

    parts
}

/// Split a segment by top-level `|` pipes, respecting nesting depth.
fn split_by_pipe(segment: &str) -> Vec<String> {
    let tokens = tokenize_with_pos_enhanced_and_roles(segment);
    let mut parts: Vec<String> = Vec::new();
    let mut last_pos = 0usize;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;

    for t in &tokens {
        match t.reconstructed_value.as_str() {
            "(" => paren_depth += 1,
            ")" => paren_depth = 0.max(paren_depth - 1),
            "{" => brace_depth += 1,
            "}" => brace_depth = 0.max(brace_depth - 1),
            _ => {}
        }

        if paren_depth == 0 && brace_depth == 0 && t.reconstructed_value == "|" {
            let chunk = segment[last_pos..t.start].trim();
            if !chunk.is_empty() {
                parts.push(chunk.to_string());
            }
            last_pos = t.end;
        }
    }

    let tail = segment[last_pos..].trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }

    parts
}

/// Handle PowerShell backtick-escaped operators: `&`& → '&&', `|`| → '||'
fn handle_backtick_escapes(cmd: &str) -> String {
    cmd.replace("`&`&", "'&&'").replace("`|`|", "'||'")
}

/// Translate `&&`/`||` connectors for legacy PowerShell (< 7).
///
/// PowerShell < 7 doesn't support `&&`/`||` natively, so we convert:
/// - `cmd1 && cmd2` → `cmd1; if ($?) { cmd2 }`
/// - `cmd1 || cmd2` → `cmd1; if (-not $?) { cmd2 }`
fn translate_for_legacy_powershell(command: &str) -> String {
    let parts = split_by_connectors(command);
    if parts.is_empty() {
        return command.to_string();
    }

    let mut script = parts[0].clone();
    let mut i = 1;
    while i < parts.len() {
        let connector = &parts[i];
        let next_cmd = if i + 1 < parts.len() {
            &parts[i + 1]
        } else {
            break;
        };

        match connector.as_str() {
            "&&" => {
                script.push_str(&format!("; if ($?) {{ {} }}", next_cmd));
            }
            "||" => {
                script.push_str(&format!("; if (-not $?) {{ {} }}", next_cmd));
            }
            _ => {}
        }
        i += 2;
    }

    script
}

/// Lint a command for unsupported segments.
pub fn lint_command(command: &str) -> LintResult {
    let registry = MappingRegistry::new();
    let source = detect_input_format(command);
    let mut unsupported: Vec<String> = Vec::new();
    let suggestions: Vec<String> = Vec::new();

    let parts = split_by_connectors(command);
    for part in &parts {
        if part == "&&" || part == "||" {
            continue;
        }
        let pipe_parts = split_by_pipe(part);
        for seg in &pipe_parts {
            let trimmed = seg.trim();
            if trimmed.is_empty() || trimmed.starts_with('(') || trimmed.starts_with('{') {
                continue;
            }

            let tokens = tokenize_with_pos_enhanced_and_roles(trimmed);
            if let Some(cmd_tok) = tokens.iter().find(|t| t.role == TokenRole::Cmd) {
                let cmd_name = &cmd_tok.reconstructed_value;
                if !registry.is_known(source, cmd_name) {
                    unsupported.push(format!("{} (unknown command: '{}')", trimmed, cmd_name));
                }
            }
        }
    }

    LintResult {
        unsupported,
        suggestions,
    }
}
