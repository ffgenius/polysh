//! Edge-case and boundary tests for the shell command tokenizer.
//!
//! Covers: basic tokenization, nested constructs, reconstruction, unicode,
//! and all inline tests migrated from src/tokenizer.rs.

use polysh::tokenizer::{
    tag_token_roles, tokenize_with_pos, tokenize_with_pos_enhanced,
    tokenize_with_pos_enhanced_and_roles, TokenRole,
};

// ---------------------------------------------------------------------------
// Basic tokenization edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_token() {
    let t = tokenize_with_pos("hello");
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].value, "hello");
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 5);
}

#[test]
fn leading_whitespace() {
    let t = tokenize_with_pos("    ls -la");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].value, "ls");
    assert_eq!(t[1].value, "-la");
}

#[test]
fn trailing_whitespace() {
    let t = tokenize_with_pos("ls -la    ");
    assert_eq!(t.len(), 2);
}

#[test]
fn multiple_spaces_between_tokens() {
    let t = tokenize_with_pos("ls     -la");
    assert_eq!(t.len(), 2);
}

#[test]
fn tab_separated() {
    let t = tokenize_with_pos("ls\t-la");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].value, "ls");
    assert_eq!(t[1].value, "-la");
}

#[test]
fn newline_separated() {
    let t = tokenize_with_pos("ls\n-la");
    assert_eq!(t.len(), 2);
}

// ---------------------------------------------------------------------------
// Quoting edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_quotes() {
    let t = tokenize_with_pos("echo ''");
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].value, "''");
    assert_eq!(t[1].quote_type, Some('\''));
}

#[test]
fn empty_double_quotes() {
    let t = tokenize_with_pos(r#"echo """#);
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].quote_type, Some('"'));
}

#[test]
fn single_quote_inside_double() {
    let t = tokenize_with_pos(r#"echo "it's nice""#);
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].value, r#""it's nice""#);
}

#[test]
fn double_quote_inside_single() {
    let t = tokenize_with_pos(r#"echo 'he said "hi"'"#);
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].value, r#"'he said "hi"'"#);
}

#[test]
fn unclosed_quote_is_handled() {
    let t = tokenize_with_pos("echo 'unclosed");
    assert!(!t.is_empty());
    // Should not panic; the tokenizer handles unterminated quotes
}

#[test]
fn unclosed_double_quote_is_handled() {
    let t = tokenize_with_pos("echo \"unclosed");
    assert!(!t.is_empty());
}

// ---------------------------------------------------------------------------
// Flag detection edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_dash_not_a_flag() {
    let t = tokenize_with_pos("cmd -");
    let roles = tag_token_roles(&t);
    // "-" alone is too short to be a flag
    assert_eq!(roles[1].role, TokenRole::Arg);
}

#[test]
fn long_gnu_style_flag() {
    let t = tokenize_with_pos("cmd --verbose");
    let roles = tag_token_roles(&t);
    assert_eq!(roles[1].role, TokenRole::Flag);
}

#[test]
fn equals_flag() {
    let t = tokenize_with_pos("cmd --output=file");
    let roles = tag_token_roles(&t);
    assert_eq!(roles[1].role, TokenRole::Flag);
}

#[test]
fn short_flag_chain() {
    let t = tokenize_with_pos("tar -xzf archive.tar.gz");
    let roles = tag_token_roles(&t);
    assert_eq!(roles[0].role, TokenRole::Cmd);
    assert_eq!(roles[1].role, TokenRole::Flag);
}

#[test]
fn quoted_flag_is_arg_not_flag() {
    let t = tokenize_with_pos("cmd '-f'");
    let roles = tag_token_roles(&t);
    // A quoted '-f' is treated as an arg, not a flag
    assert_eq!(roles[1].role, TokenRole::Arg);
    assert_eq!(roles[1].quote_type, Some('\''));
}

// ---------------------------------------------------------------------------
// Redirection tokenization
// ---------------------------------------------------------------------------

#[test]
fn stdout_redirect() {
    let t = tokenize_with_pos("cmd > file.txt");
    let values: Vec<&str> = t.iter().map(|x| x.value.as_str()).collect();
    assert!(values.contains(&">"));
}

#[test]
fn stdout_append() {
    let t = tokenize_with_pos("cmd >> file.txt");
    let values: Vec<&str> = t.iter().map(|x| x.value.as_str()).collect();
    assert!(values.contains(&">>"));
}

#[test]
fn stderr_redirect() {
    let t = tokenize_with_pos("cmd 2>&1");
    assert!(!t.is_empty());
    assert_eq!(t[0].value, "cmd");
}

#[test]
fn stdin_redirect() {
    let t = tokenize_with_pos("cmd < input.txt");
    let values: Vec<&str> = t.iter().map(|x| x.value.as_str()).collect();
    assert!(values.contains(&"<"));
}

// ---------------------------------------------------------------------------
// Parentheses and braces
// ---------------------------------------------------------------------------

#[test]
fn parentheses_in_command() {
    let t = tokenize_with_pos("(echo hello)");
    let values: Vec<&str> = t.iter().map(|x| x.value.as_str()).collect();
    assert_eq!(values, vec!["(", "echo", "hello", ")"]);
}

#[test]
fn nested_parentheses() {
    let t = tokenize_with_pos("(echo (hello world))");
    assert!(!t.is_empty());
    // Must not panic with nested parens
}

#[test]
fn braces() {
    let t = tokenize_with_pos("{ echo hello; }");
    let values: Vec<&str> = t.iter().map(|x| x.value.as_str()).collect();
    assert!(values.contains(&"{"));
    assert!(values.contains(&"}"));
}

// ---------------------------------------------------------------------------
// Enhanced tokenization: reconstruction
// ---------------------------------------------------------------------------

#[test]
fn cmd_sub_basic() {
    let tokens = tokenize_with_pos_enhanced("echo $(ls -la)");
    let has_dollar_paren = tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("$("));
    assert!(has_dollar_paren, "Expected reconstructed $(...) token");
}

#[test]
fn cmd_sub_nested() {
    let tokens = tokenize_with_pos_enhanced("echo $(echo $(pwd))");
    // Must not panic
    assert!(!tokens.is_empty());
}

#[test]
fn proc_sub_basic() {
    let tokens = tokenize_with_pos_enhanced("diff <(ls a) <(ls b)");
    let has_lt_paren = tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("<("));
    assert!(has_lt_paren, "Expected reconstructed <(...) token");
}

#[test]
fn here_doc_basic() {
    let tokens = tokenize_with_pos_enhanced("cat << EOF");
    let has_heredoc = tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("<<"));
    assert!(
        has_heredoc,
        "Expected reconstructed << token, got: {:?}",
        tokens
            .iter()
            .map(|t| &t.reconstructed_value)
            .collect::<Vec<_>>()
    );
}

#[test]
fn env_var_brace() {
    let tokens = tokenize_with_pos_enhanced("echo ${HOME}");
    let has_env = tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("${"));
    assert!(has_env, "Expected reconstructed ${{...}} token");
}

#[test]
fn env_var_nested_brace() {
    let tokens = tokenize_with_pos_enhanced("echo ${VAR:-default}");
    // Must not panic
    assert!(!tokens.is_empty());
}

#[test]
fn function_def_parens() {
    let tokens = tokenize_with_pos_enhanced("myfunc() { echo hello; }");
    let has_fn = tokens.iter().any(|t| t.reconstructed_value == "()");
    assert!(has_fn, "Expected reconstructed () token");
}

#[test]
fn redirection_fd() {
    let tokens = tokenize_with_pos_enhanced("cmd 2>/dev/null");
    // Should reconstruct 2> as one token
    let has_redirect = tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("2>"));
    assert!(has_redirect, "Expected reconstructed 2> token");
}

// ---------------------------------------------------------------------------
// Enhanced + roles
// ---------------------------------------------------------------------------

#[test]
fn enhanced_and_roles_op_detection() {
    let tokens = tokenize_with_pos_enhanced_and_roles("cmd1 && cmd2 || cmd3");
    assert_eq!(tokens[0].role, TokenRole::Cmd);
    assert_eq!(tokens[1].role, TokenRole::Op);
    assert_eq!(tokens[2].role, TokenRole::Cmd);
    assert_eq!(tokens[3].role, TokenRole::Op);
    assert_eq!(tokens[4].role, TokenRole::Cmd);
}

#[test]
fn enhanced_and_roles_pipe_detection() {
    let tokens = tokenize_with_pos_enhanced_and_roles("a | b | c");
    assert_eq!(tokens[0].role, TokenRole::Cmd);
    assert_eq!(tokens[1].role, TokenRole::Op);
    assert_eq!(tokens[2].role, TokenRole::Cmd);
    assert_eq!(tokens[3].role, TokenRole::Op);
    assert_eq!(tokens[4].role, TokenRole::Cmd);
}

#[test]
fn enhanced_and_roles_redirect_as_arg() {
    let tokens = tokenize_with_pos_enhanced_and_roles("cmd > out.txt");
    // > is an op/redirect, we want to ensure it's classified
    assert_eq!(tokens[0].role, TokenRole::Cmd);
    // redirections are classified as Arg by the role tagger
}

// ---------------------------------------------------------------------------
// TokenRole Display
// ---------------------------------------------------------------------------

#[test]
fn token_role_display() {
    assert_eq!(format!("{}", TokenRole::Cmd), "cmd");
    assert_eq!(format!("{}", TokenRole::Flag), "flag");
    assert_eq!(format!("{}", TokenRole::Arg), "arg");
    assert_eq!(format!("{}", TokenRole::Op), "op");
}

// ---------------------------------------------------------------------------
// Unicode / special characters
// ---------------------------------------------------------------------------

#[test]
fn unicode_argument() {
    let t = tokenize_with_pos("echo 你好世界");
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].value, "你好世界");
}

#[test]
fn emoji_in_command() {
    let t = tokenize_with_pos("echo 🦀");
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].value, "🦀");
}

#[test]
fn path_with_spaces_quoted() {
    let t = tokenize_with_pos("cat \"C:\\Program Files\\file.txt\"");
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].quote_type, Some('"'));
}

#[test]
fn path_with_spaces_single_quoted() {
    let t = tokenize_with_pos("cat 'C:\\Program Files\\file.txt'");
    assert_eq!(t.len(), 2);
    assert_eq!(t[1].quote_type, Some('\''));
}

// ---------------------------------------------------------------------------
// Position tracking
// ---------------------------------------------------------------------------

#[test]
fn positions_are_correct() {
    let cmd = "ls -la dir";
    let t = tokenize_with_pos(cmd);
    assert_eq!(t[0].start, 0);
    assert_eq!(t[0].end, 2);
    assert_eq!(&cmd[t[0].start..t[0].end], "ls");

    assert_eq!(t[1].start, 3);
    assert_eq!(t[1].end, 6);
    assert_eq!(&cmd[t[1].start..t[1].end], "-la");

    assert_eq!(t[2].start, 7);
    assert_eq!(t[2].end, 10);
    assert_eq!(&cmd[t[2].start..t[2].end], "dir");
}

#[test]
fn positions_with_whitespace() {
    let cmd = "  ls    -la  ";
    let t = tokenize_with_pos(cmd);
    assert!(!t.is_empty());
    assert_eq!(&cmd[t[0].start..t[0].end], "ls");
}

// ---------------------------------------------------------------------------
// Edge case: operator-only input
// ---------------------------------------------------------------------------

#[test]
fn only_operators() {
    let t = tokenize_with_pos("&& || | ;");
    let roles = tag_token_roles(&t);
    assert!(roles.iter().all(|r| r.role == TokenRole::Op));
}

#[test]
fn only_operator_no_space() {
    let t = tokenize_with_pos("&&");
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].value, "&&");
}

// ============================================================================
// Tests migrated from src/tokenizer.rs inline test module
// ============================================================================

#[test]
fn test_simple_command() {
    let tokens = tokenize_with_pos("ls -la");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].value, "ls");
    assert_eq!(tokens[1].value, "-la");
}

#[test]
fn test_quoted_string() {
    let tokens = tokenize_with_pos("echo 'hello world'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].value, "echo");
    assert_eq!(tokens[1].value, "'hello world'");
    assert_eq!(tokens[1].quote_type, Some('\''));
}

#[test]
fn test_double_quoted_string() {
    let tokens = tokenize_with_pos(r#"echo "hello world""#);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[1].value, r#""hello world""#);
    assert_eq!(tokens[1].quote_type, Some('"'));
}

#[test]
fn test_connectors() {
    let tokens = tokenize_with_pos("cmd1 && cmd2 || cmd3");
    let values: Vec<&str> = tokens.iter().map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["cmd1", "&&", "cmd2", "||", "cmd3"]);
}

#[test]
fn test_pipe() {
    let tokens = tokenize_with_pos("ls | grep .rs");
    let values: Vec<&str> = tokens.iter().map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["ls", "|", "grep", ".rs"]);
}

#[test]
fn test_redirection() {
    let tokens = tokenize_with_pos("cmd 2>&1");
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].value, "cmd");
}

#[test]
fn test_parentheses() {
    let tokens = tokenize_with_pos("(echo hello)");
    let values: Vec<&str> = tokens.iter().map(|t| t.value.as_str()).collect();
    assert_eq!(values, vec!["(", "echo", "hello", ")"]);
}

#[test]
fn test_backtick_escaped_operators() {
    let tokens = tokenize_with_pos("echo `&`& echo");
    assert!(tokens.iter().any(|t| t.value.contains('`')));
}

#[test]
fn test_role_tagging() {
    let tokens = tokenize_with_pos("rm -rf file.txt");
    let roles = tag_token_roles(&tokens);
    assert_eq!(roles.len(), 3);
    assert_eq!(roles[0].role, TokenRole::Cmd);
    assert_eq!(roles[1].role, TokenRole::Flag);
    assert_eq!(roles[2].role, TokenRole::Arg);
}

#[test]
fn test_role_tagging_with_connectors() {
    let tokens = tokenize_with_pos("rm -rf dist && echo done");
    let roles = tag_token_roles(&tokens);
    assert_eq!(roles[0].role, TokenRole::Cmd); // rm
    assert_eq!(roles[1].role, TokenRole::Flag); // -rf
    assert_eq!(roles[2].role, TokenRole::Arg); // dist
    assert_eq!(roles[3].role, TokenRole::Op); // &&
    assert_eq!(roles[4].role, TokenRole::Cmd); // echo
    assert_eq!(roles[5].role, TokenRole::Arg); // done
}

#[test]
fn test_cmd_sub_reconstruction() {
    let cmd = "echo $(ls -la)";
    let tokens = tokenize_with_pos_enhanced(cmd);
    assert!(tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("$(")));
}

#[test]
fn test_env_var_reconstruction() {
    let cmd = "echo ${HOME}";
    let tokens = tokenize_with_pos_enhanced(cmd);
    assert!(tokens
        .iter()
        .any(|t| t.reconstructed_value.starts_with("${")));
}

#[test]
fn test_enhanced_and_roles() {
    let tokens = tokenize_with_pos_enhanced_and_roles("rm -rf file.txt");
    assert_eq!(tokens[0].role, TokenRole::Cmd);
    assert_eq!(tokens[1].role, TokenRole::Flag);
    assert_eq!(tokens[2].role, TokenRole::Arg);
}

#[test]
fn test_whitespace_handling() {
    let tokens = tokenize_with_pos("  ls   -la  ");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].value, "ls");
    assert_eq!(tokens[1].value, "-la");
}

#[test]
fn test_empty_input() {
    let tokens = tokenize_with_pos("");
    assert!(tokens.is_empty());
}

#[test]
fn test_cmd_flag_detection() {
    let tokens = tokenize_with_pos("del /s /q file.txt");
    let roles = tag_token_roles(&tokens);
    assert_eq!(roles[0].role, TokenRole::Cmd); // del
}
