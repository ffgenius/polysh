//! Shell command tokenizer.
//!
//! Splits a shell command string into typed tokens (commands, flags, arguments,
//! operators), preserving original positions and respecting shell quoting rules.
//!
//! Supports Unix (bash/zsh/fish), PowerShell, and CMD syntax conventions
//! including backtick-escaped operators, command substitution, here-documents,
//! process substitution, and environment variable expansion.

/// A raw token produced by the initial scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub start: usize,
    pub end: usize, // exclusive
    pub quote_type: Option<char>, // ' or " if quoted, None otherwise
}

/// An enhanced token with original text preservation and reconstructed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnhancedToken {
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub original_text: String,
    pub reconstructed_value: String,
    pub needs_reconstruction: bool,
    pub quote_type: Option<char>,
}

/// The role a token plays in a shell command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenRole {
    Cmd,
    Flag,
    Arg,
    Op,
}

impl std::fmt::Display for TokenRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenRole::Cmd => write!(f, "cmd"),
            TokenRole::Flag => write!(f, "flag"),
            TokenRole::Arg => write!(f, "arg"),
            TokenRole::Op => write!(f, "op"),
        }
    }
}

/// A token with its semantic role assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleToken {
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub original_text: String,
    pub reconstructed_value: String,
    pub needs_reconstruction: bool,
    pub quote_type: Option<char>,
    pub role: TokenRole,
}

// ---------------------------------------------------------------------------
// Character classification helpers
// ---------------------------------------------------------------------------

fn is_operator_start(ch: char) -> bool {
    matches!(ch, '<' | '>' | '|' | '&' | ';' | '(' | ')' | '{' | '}')
}

fn is_operator_at(s: &str, pos: usize) -> bool {
    const OPERATORS: &[&str] = &[
        "&&", "||", "|&", "<<", ">>", "|", ";", "<", ">", "(", ")", "{", "}",
    ];
    OPERATORS.iter().any(|op| s[pos..].starts_with(op))
}

fn extract_operator_at(s: &str, pos: usize) -> String {
    const OPERATORS: &[&str] = &[
        "&&", "||", "|&", "<<", ">>", "|", ";", "<", ">", "(", ")", "{", "}",
    ];
    for op in OPERATORS {
        if s[pos..].starts_with(op) {
            return op.to_string();
        }
    }
    s[pos..pos + 1].to_string()
}

// ---------------------------------------------------------------------------
// Core tokenizer
// ---------------------------------------------------------------------------

/// Tokenize a shell command string into `Token` structs with position info.
///
/// This is the foundational tokenizer. It handles:
/// - Quoted strings (`'...'` and `"..."`)
/// - Backslash-escaped operators (`\&&`)
/// - PowerShell backtick-escaped operators (`` `&`& ``)
/// - Shell operators (`&&`, `||`, `|`, `;`, `>`, `>>`, `<`, `<<`, `(`, `)`, `{`, `}`)
pub fn tokenize_with_pos(cmd: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = cmd.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        let start = i;
        let mut value = String::new();
        let mut quote_type: Option<char> = None;

        // Quoted strings
        if chars[i] == '\'' || chars[i] == '"' {
            quote_type = Some(chars[i]);
            let quote_char = chars[i];
            value.push(chars[i]);
            i += 1; // skip opening quote

            while i < len && chars[i] != quote_char {
                value.push(chars[i]);
                i += 1;
            }

            if i < len {
                value.push(chars[i]); // closing quote
                i += 1;
            }
        }
        // Backslash-escaped operator: \&&, \|| etc.
        else if chars[i] == '\\'
            && i + 1 < len
            && is_operator_start(chars[i + 1])
        {
            value.push(chars[i]);
            value.push(chars[i + 1]);
            i += 2;
        }
        // PowerShell backtick-escaped: `&`&  or  `|`|
        else if chars[i] == '`'
            && i + 1 < len
            && is_operator_start(chars[i + 1])
        {
            if chars[i + 1] == '&'
                && i + 2 < len
                && chars[i + 2] == '`'
                && i + 3 < len
                && chars[i + 3] == '&'
            {
                // `&`&
                value.push(chars[i]);
                value.push(chars[i + 1]);
                value.push(chars[i + 2]);
                value.push(chars[i + 3]);
                i += 4;
            } else if chars[i + 1] == '|'
                && i + 2 < len
                && chars[i + 2] == '`'
                && i + 3 < len
                && chars[i + 3] == '|'
            {
                // `|`|
                value.push(chars[i]);
                value.push(chars[i + 1]);
                value.push(chars[i + 2]);
                value.push(chars[i + 3]);
                i += 4;
            } else {
                // Single backtick-escaped operator: `&, `|
                value.push(chars[i]);
                value.push(chars[i + 1]);
                i += 2;
            }
        }
        // Shell operators
        else if is_operator_at(cmd, i) {
            let op = extract_operator_at(cmd, i);
            let op_len = op.len();
            value = op;
            i += op_len;
        }
        // Regular tokens: non-whitespace, non-operator characters
        else {
            while i < len
                && !chars[i].is_ascii_whitespace()
                && !is_operator_start(chars[i])
            {
                value.push(chars[i]);
                i += 1;
            }
        }

        // Safety: if we didn't advance, force-advance one character
        if i == start {
            value.push(chars[i]);
            i += 1;
        }

        if !value.is_empty() {
            tokens.push(Token {
                value,
                start,
                end: i,
                quote_type,
            });
        }
    }

    tokens
}

// ---------------------------------------------------------------------------
// Enhanced tokenizer with shell construct reconstruction
// ---------------------------------------------------------------------------

/// Enhanced tokenization that reconstructs shell construct boundaries.
pub fn tokenize_with_pos_enhanced(cmd: &str) -> Vec<EnhancedToken> {
    let basic_tokens = tokenize_with_pos(cmd);
    let enhanced: Vec<EnhancedToken> = basic_tokens
        .iter()
        .map(|t| EnhancedToken {
            value: t.value.clone(),
            start: t.start,
            end: t.end,
            original_text: cmd[t.start..t.end].to_string(),
            reconstructed_value: t.value.clone(),
            needs_reconstruction: false,
            quote_type: t.quote_type,
        })
        .collect();

    reconstruct_special_constructs(enhanced, cmd)
}

fn reconstruct_special_constructs(tokens: Vec<EnhancedToken>, _cmd: &str) -> Vec<EnhancedToken> {
    let result = reconstruct_command_substitution(tokens);
    let result = reconstruct_here_docs(result);
    let result = reconstruct_process_substitution(result);
    let result = reconstruct_function_defs(result);
    let result = reconstruct_env_vars(result);
    let result = reconstruct_redirections(result);
    reconstruct_escaped_operators(result)
}

// ---- Command substitution: $(...) ----

fn reconstruct_command_substitution(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value == "$" && i + 1 < tokens.len() && tokens[i + 1].value == "(" {
            let mut paren_depth = 1i32;
            let mut j = i + 2;
            while j < tokens.len() && paren_depth > 0 {
                match tokens[j].value.as_str() {
                    "(" => paren_depth += 1,
                    ")" => paren_depth -= 1,
                    _ => {}
                }
                j += 1;
            }

            if paren_depth == 0 {
                let start = tokens[i].start;
                let end = tokens[j - 1].end;
                let original_text = cmd_from_range(&tokens, i, j);
                let mut reconstructed = String::from("$(");
                for k in i + 2..j - 1 {
                    reconstructed.push_str(&tokens[k].value);
                    reconstructed.push(' ');
                }
                reconstructed = reconstructed.trim().to_string();
                reconstructed.push(')');

                result.push(EnhancedToken {
                    value: "$(".to_string(),
                    start,
                    end,
                    original_text,
                    reconstructed_value: reconstructed,
                    needs_reconstruction: false,
                    quote_type: None,
                });
                i = j - 1;
            } else {
                result.push(tokens[i].clone());
            }
        } else {
            result.push(tokens[i].clone());
        }
        i += 1;
    }

    result
}

// ---- Here-document: << 'EOF' ----

fn reconstruct_here_docs(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value == "<" && i + 1 < tokens.len() && tokens[i + 1].value == "<" {
            let start = tokens[i].start;
            let mut end = tokens[i + 1].end;
            let mut original = tokens[i].original_text.clone() + &tokens[i + 1].original_text;
            let mut reconstructed = String::from("<<");

            // Include the delimiter if present
            let mut skip = 1; // skip the second <
            if i + 2 < tokens.len() {
                let delim = &tokens[i + 2];
                end = delim.end;
                original.push_str(&delim.original_text);
                reconstructed.push_str(&delim.value);
                skip = 2;
            }

            result.push(EnhancedToken {
                value: "<<".to_string(),
                start,
                end,
                original_text: original,
                reconstructed_value: reconstructed,
                needs_reconstruction: false,
                quote_type: None,
            });
            i += skip;
        } else {
            result.push(tokens[i].clone());
        }
        i += 1;
    }

    result
}

// ---- Process substitution: <(...) ----

fn reconstruct_process_substitution(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value == "<"
            && i + 1 < tokens.len()
            && tokens[i + 1].value == "("
        {
            let mut paren_depth = 1i32;
            let mut j = i + 2;
            while j < tokens.len() && paren_depth > 0 {
                match tokens[j].value.as_str() {
                    "(" => paren_depth += 1,
                    ")" => paren_depth -= 1,
                    _ => {}
                }
                j += 1;
            }

            if paren_depth == 0 {
                let start = tokens[i].start;
                let end = tokens[j - 1].end;
                let original_text = cmd_from_range(&tokens, i, j);
                let mut reconstructed = String::from("<(");
                for k in i + 2..j - 1 {
                    reconstructed.push_str(&tokens[k].value);
                    reconstructed.push(' ');
                }
                reconstructed.push(')');

                result.push(EnhancedToken {
                    value: "<(".to_string(),
                    start,
                    end,
                    original_text,
                    reconstructed_value: reconstructed,
                    needs_reconstruction: false,
                    quote_type: None,
                });
                i = j - 1;
            } else {
                result.push(tokens[i].clone());
            }
        } else {
            result.push(tokens[i].clone());
        }
        i += 1;
    }

    result
}

// ---- Function definitions: () ----

fn reconstruct_function_defs(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value == "(" && i + 1 < tokens.len() && tokens[i + 1].value == ")" {
            result.push(EnhancedToken {
                value: "()".to_string(),
                start: tokens[i].start,
                end: tokens[i + 1].end,
                original_text: tokens[i].original_text.clone()
                    + &tokens[i + 1].original_text,
                reconstructed_value: "()".to_string(),
                needs_reconstruction: false,
                quote_type: None,
            });
            i += 2;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

// ---- Environment variables: ${...} ----

fn reconstruct_env_vars(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value == "$" && i + 1 < tokens.len() && tokens[i + 1].value.starts_with('{')
        {
            let mut brace_depth = 1i32;
            let mut j = i + 1;
            // Count braces in the token that starts with '{'
            for ch in tokens[i + 1].value.chars() {
                if ch == '{' {
                    brace_depth += 1;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            while brace_depth > 0 && j + 1 < tokens.len() {
                j += 1;
                for ch in tokens[j].value.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                    }
                    if ch == '}' {
                        brace_depth -= 1;
                    }
                }
            }

            let start = tokens[i].start;
            let end = tokens[j].end;
            let original_text = cmd_from_range(&tokens, i, j + 1);
            let mut reconstructed = String::from("${");
            for k in i + 2..j {
                reconstructed.push_str(&tokens[k].value);
                reconstructed.push(' ');
            }
            reconstructed = reconstructed.trim().to_string();
            reconstructed.push('}');

            result.push(EnhancedToken {
                value: "${".to_string(),
                start,
                end,
                original_text,
                reconstructed_value: reconstructed,
                needs_reconstruction: false,
                quote_type: None,
            });
            i = j;
        } else {
            result.push(tokens[i].clone());
        }
        i += 1;
    }

    result
}

// ---- Redirections: 2>, 2>>, 2>&1, &> ----

fn reconstruct_redirections(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // Check for patterns like "2" followed by ">" or ">>" or ">&"
        if tokens[i].value.chars().all(|c| c.is_ascii_digit())
            && i + 1 < tokens.len()
            && is_redirection_token(&tokens[i + 1].value)
        {
            let combined_val = format!("{}{}", tokens[i].value, tokens[i + 1].value);
            result.push(EnhancedToken {
                value: combined_val.clone(),
                start: tokens[i].start,
                end: tokens[i + 1].end,
                original_text: tokens[i].original_text.clone()
                    + &tokens[i + 1].original_text,
                reconstructed_value: combined_val,
                needs_reconstruction: false,
                quote_type: None,
            });
            i += 2;
        } else if is_redirection_token(&tokens[i].value) {
            result.push(tokens[i].clone());
            i += 1;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

// ---- Escaped operators: \&&, \|| ----

fn reconstruct_escaped_operators(tokens: Vec<EnhancedToken>) -> Vec<EnhancedToken> {
    let mut result: Vec<EnhancedToken> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].value.starts_with('\\')
            && i + 1 < tokens.len()
            && is_operator_start(tokens[i + 1].value.chars().next().unwrap_or(' '))
        {
            let combined = format!("{}{}", tokens[i].value, tokens[i + 1].value);
            result.push(EnhancedToken {
                value: combined.clone(),
                start: tokens[i].start,
                end: tokens[i + 1].end,
                original_text: tokens[i].original_text.clone()
                    + &tokens[i + 1].original_text,
                reconstructed_value: combined,
                needs_reconstruction: false,
                quote_type: None,
            });
            i += 2;
        } else {
            result.push(tokens[i].clone());
            i += 1;
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Role tagging
// ---------------------------------------------------------------------------

const OPS: &[&str] = &["&&", "||", "|", ";", "|&"];

fn is_redirection_token(val: &str) -> bool {
    // Matches: > >> < 2> 2>> 2>&1 &> &>> etc.
    let bytes = val.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    // Check character by character
    let mut has_digit_prefix = false;
    for (idx, &b) in bytes.iter().enumerate() {
        match b {
            b'0'..=b'9' => {
                if idx == 0 {
                    has_digit_prefix = true;
                } else if !has_digit_prefix {
                    return false;
                }
            }
            b'>' | b'<' | b'&' => {
                // valid redirect chars
            }
            _ => return false,
        }
    }
    // Must contain at least one redirect symbol
    bytes.iter().any(|&b| b == b'>' || b == b'<' || b == b'&')
}

/// Assign a semantic role to each token: Cmd, Flag, Arg, or Op.
pub fn tag_token_roles(tokens: &[Token]) -> Vec<RoleToken> {
    let mut out: Vec<RoleToken> = Vec::new();
    let mut expect_cmd = true;

    for t in tokens {
        let enhanced = EnhancedToken {
            value: t.value.clone(),
            start: t.start,
            end: t.end,
            original_text: t.value.clone(),
            reconstructed_value: t.value.clone(),
            needs_reconstruction: false,
            quote_type: t.quote_type,
        };

        if OPS.contains(&t.value.as_str()) {
            out.push(RoleToken {
                value: enhanced.value,
                start: enhanced.start,
                end: enhanced.end,
                original_text: enhanced.original_text,
                reconstructed_value: enhanced.reconstructed_value,
                needs_reconstruction: false,
                quote_type: enhanced.quote_type,
                role: TokenRole::Op,
            });
            expect_cmd = true;
            continue;
        }

        if is_redirection_token(&t.value) {
            out.push(RoleToken {
                value: enhanced.value,
                start: enhanced.start,
                end: enhanced.end,
                original_text: enhanced.original_text,
                reconstructed_value: enhanced.reconstructed_value,
                needs_reconstruction: false,
                quote_type: enhanced.quote_type,
                role: TokenRole::Arg,
            });
            continue;
        }

        if expect_cmd {
            out.push(RoleToken {
                value: enhanced.value,
                start: enhanced.start,
                end: enhanced.end,
                original_text: enhanced.original_text,
                reconstructed_value: enhanced.reconstructed_value,
                needs_reconstruction: false,
                quote_type: enhanced.quote_type,
                role: TokenRole::Cmd,
            });
            expect_cmd = false;
            continue;
        }

        if t.value.starts_with('-')
            && t.value.len() > 1
            && t.quote_type.is_none()
        {
            out.push(RoleToken {
                value: enhanced.value,
                start: enhanced.start,
                end: enhanced.end,
                original_text: enhanced.original_text,
                reconstructed_value: enhanced.reconstructed_value,
                needs_reconstruction: false,
                quote_type: enhanced.quote_type,
                role: TokenRole::Flag,
            });
        } else {
            out.push(RoleToken {
                value: enhanced.value,
                start: enhanced.start,
                end: enhanced.end,
                original_text: enhanced.original_text,
                reconstructed_value: enhanced.reconstructed_value,
                needs_reconstruction: false,
                quote_type: enhanced.quote_type,
                role: TokenRole::Arg,
            });
        }
    }

    out
}

/// Enhanced tokenization with reconstruction AND role tagging in one pass.
pub fn tokenize_with_pos_enhanced_and_roles(cmd: &str) -> Vec<RoleToken> {
    let enhanced = tokenize_with_pos_enhanced(cmd);
    assign_roles_from_enhanced(&enhanced)
}

fn assign_roles_from_enhanced(tokens: &[EnhancedToken]) -> Vec<RoleToken> {
    let mut out: Vec<RoleToken> = Vec::new();
    let mut expect_cmd = true;
    // We use the previous token's info to decide

    for t in tokens {
        let value = &t.reconstructed_value;
        let role;

        if OPS.contains(&value.as_str()) {
            role = TokenRole::Op;
            expect_cmd = true;
        } else if is_redirection_token(value) {
            role = TokenRole::Arg;
        } else if expect_cmd {
            role = TokenRole::Cmd;
            expect_cmd = false;
        } else if value.starts_with('-')
            && value.len() > 1
            && t.quote_type.is_none()
        {
            role = TokenRole::Flag;
        } else {
            role = TokenRole::Arg;
        }

        out.push(RoleToken {
            value: t.value.clone(),
            start: t.start,
            end: t.end,
            original_text: t.original_text.clone(),
            reconstructed_value: t.reconstructed_value.clone(),
            needs_reconstruction: t.needs_reconstruction,
            quote_type: t.quote_type,
            role,
        });
    }

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cmd_from_range(tokens: &[EnhancedToken], start: usize, end: usize) -> String {
    tokens[start..end]
        .iter()
        .map(|t| t.original_text.as_str())
        .collect::<Vec<_>>()
        .join("")
}
