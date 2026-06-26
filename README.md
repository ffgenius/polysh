# polysh

> Bidirectional shell command translator — **Unix ⇄ PowerShell ⇄ CMD**

[![Crates.io](https://img.shields.io/crates/v/polysh)](https://crates.io/crates/polysh)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[English](README.md) | [中文](README.zh-CH.md)

**polysh** is a pure-Rust shell command translation library. It translates commands between three shell dialects:

```
Unix (bash / zsh / fish)   ⇄   PowerShell   ⇄   CMD (Command Prompt)
```

Zero external dependencies. All mapping data is embedded at compile time.

## Use Case

You have a configuration file containing a `sh` command:

```yaml
# app-config.yml
build: "rm -rf dist && npm run build"
```

Your CLI tool needs to run the equivalent on Windows. Use polysh:

```rust
use polysh::detector::detect_shell;
use polysh::mappings::{MappingRegistry, Dialect};
use polysh::translator::{detect_input_format, translate_with_registry};

let cmd = "rm -rf dist && npm run build";

let source = detect_input_format(cmd);         // → Dialect::Unix
let shell  = detect_shell();                    // current platform
let reg    = MappingRegistry::new();

let translated = translate_with_registry(
    cmd, source, shell.target, &shell, &reg,
);

// Linux:   "rm -rf dist && npm run build"
// Windows: "Remove-Item -Recurse -Force dist; if ($?) { npm run build }"
```

## Quick Start

```toml
[dependencies]
polysh = "0.0.1"
```

### Simplest usage: auto-detect + translate

```rust
use polysh::detector::ShellInfo;
use polysh::mappings::Dialect;
use polysh::translator::translate_command;

let shell = ShellInfo {
    dialect: Dialect::PowerShell,
    supports_conditional_connectors: true,
    needs_unix_translation: true,
    target: Dialect::PowerShell,
    version: Some(7),
};

let result = translate_command("rm -rf dist && echo done", &shell);
// → "Remove-Item -Recurse -Force dist; if ($?) { Write-Host done }"
```

### Explicit source and target

```rust
use polysh::mappings::{MappingRegistry, Dialect};
use polysh::translator::translate_with_registry;

let reg = MappingRegistry::new();

// Unix → PowerShell
let ps = translate_with_registry(
    "grep -in 'error' log.txt",
    Dialect::Unix,
    Dialect::PowerShell,
    &shell, &reg,
);
// → "Select-String -CaseSensitive:$false -LineNumber 'error' log.txt"

// PowerShell → Unix
let unix = translate_with_registry(
    "Get-ChildItem -Recurse -Filter '*.rs'",
    Dialect::PowerShell,
    Dialect::Unix,
    &shell, &reg,
);
// → "find . -name '*.rs'"
```

### Manual dialect detection

```rust
use polysh::translator::detect_input_format;

assert_eq!(detect_input_format("rm -rf dist"), Dialect::Unix);
assert_eq!(detect_input_format("Remove-Item -Force"), Dialect::PowerShell);
assert_eq!(detect_input_format("del /s /q dist"), Dialect::Cmd);
```

### Lint: find untranslatable segments

```rust
use polysh::translator::lint_command;

let result = lint_command("rm -rf dist");
assert!(result.unsupported.is_empty());  // ✅ fully translatable

let result = lint_command("unknown_cmd --flag");
assert!(!result.unsupported.is_empty()); // ❌ has untranslatable segments
```

## API Reference

```rust
// Core translation
polysh::translator::translate_command(cmd, &shell) -> String
polysh::translator::translate_with_registry(cmd, from, to, &shell, &reg) -> String

// Format detection
polysh::translator::detect_input_format(cmd) -> Dialect
polysh::translator::lint_command(cmd) -> LintResult

// Shell detection
polysh::detector::detect_shell() -> ShellInfo

// Tokenizer (for standalone use)
polysh::tokenizer::tokenize_with_pos(cmd) -> Vec<Token>
polysh::tokenizer::tokenize_with_pos_enhanced_and_roles(cmd) -> Vec<RoleToken>

// Registry (for custom translation logic)
polysh::mappings::MappingRegistry::new()
polysh::mappings::MappingRegistry::lookup_cmd(dialect, name) -> Option<&CommandMapping>
polysh::mappings::MappingRegistry::translate_flag(from, cmd, flag, to) -> Option<&str>
```

## Translation Directions

All 6 directions have static mappings (command name + flags). Complex commands get dynamic translators:

| Direction | Static | Dynamic | Typical use case |
|-----------|:---:|:---:|------|
| Unix → PowerShell | ✅ | 23 commands | Linux config → Windows exec |
| PowerShell → Unix | ✅ | 20 commands | PS script → Linux exec |
| Unix → CMD | ✅ | 6 commands | Linux config → CMD exec |
| CMD → Unix | ✅ | 7 commands | Batch file → Linux exec |
| PowerShell → CMD | ✅ | 6 commands | PS → pure CMD |
| CMD → PowerShell | ✅ | 7 commands | Batch file → PowerShell |

### Static mapping coverage (~100 commands)

**File operations**: `rm` `ls` `cp` `mv` `mkdir` `touch` `cat` `rmdir`

**Text processing**: `grep` `echo` `sort` `uniq` `wc` `head` `tail` `awk` `sed` `cut` `tr` `diff` `tee`

**System info**: `ps` `kill` `top` `df` `du` `free` `uptime` `whoami` `hostname` `date` `clear` `which` `uname`

**Service management**: `systemctl` `shutdown` `reboot`

**File system**: `find` `chmod` `chown` `chgrp` `ln` `stat` `dirname` `basename` `realpath`

**Archives**: `tar` `gzip` `gunzip` `zip` `unzip` `bzip2` `bunzip2`

**Network**: `curl` `wget` `ping` `ssh` `ifconfig` `netstat` `traceroute` `dig` `nslookup` `route`

**Package managers / build tools**: `apt` `brew` `npm` `pnpm` `yarn` `pip` `cargo` `make` `cmake` `gcc` `g++`

**VCS / DevOps**: `git` `docker` `kubectl` `terraform` `ansible` `svn` `vagrant`

**Databases**: `mysql` `psql` `conda`

**User management**: `sudo` `useradd` `userdel`

### Dynamic translation highlights

| Unix command | Special handling |
|-------------|-----------------|
| `find -name -delete` | Assembles multi-cmdlet pipeline |
| `sed 's/old/new/'` | Converts to `-replace` operator |
| `awk '{print $N}'` | `ForEach-Object` + `Split` |
| `cut -d -f` | `ForEach-Object` + `Split` |
| `tr 'a' 'b'` | `Replace` / `-replace` |
| `systemctl start/stop/enable...` | PS/CMD service commands |
| `chmod 755` / `chown` | `icacls` permission mapping (approximate) |
| `ln -s` / `mklink` | Auto-reverses argument order |
| `sudo` / `runas` | Privilege elevation conversion |

## Environment Variables

| Variable | Effect |
|----------|--------|
| `POLYSH_SHELL` | Override current shell type: `unix` / `powershell` / `ps` / `cmd` |

If not set, auto-detection is used.

## Module Structure

```
src/
├── lib.rs           # Library entry point, module declarations
├── tokenizer.rs     # Command string → typed token sequence
├── translator.rs    # Core engine: split → translate → reassemble
├── detector.rs      # Detect current shell environment
└── mappings/
    ├── mod.rs       # MappingRegistry: O(1) bidirectional lookup table
    ├── data.rs      # Static data: ~100 CommandMappings
    └── dynamic.rs   # Dynamic translator: handles complex argument structures
```

Data flow: `command string → tokenize → split_by_connectors → split_by_pipe → translate_segment → reassemble`

See [`docs/`](docs/) for per-file code documentation.

## Known Limitations

- **Command name collisions**: `Get-ChildItem` maps to both `ls` and `umask`; `dir` maps to both `ls` and `stat` (last HashMap insert wins)
- **`icacls` permission mapping**: Windows ACL is richer than Unix octal; conversion is approximate
- **PS `ForEach-Object` reversal**: Relies on substring matching of script block content, not AST parsing
- **CMD `%VAR%` variables**: The tokenizer does not handle them specially; they are treated as plain text

## License

[MIT](./LICENSE)
