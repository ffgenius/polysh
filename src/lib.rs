//! # polysh
//!
//! A bidirectional shell command translator — Unix ↔ PowerShell ↔ CMD.
//!
//! `polysh` translates shell commands between Unix-style shells (bash, zsh, fish),
//! PowerShell, and Windows CMD. It handles command names, flags, connectors (`&&`, `||`),
//! pipes (`|`), and complex dynamic commands like `find`, `sed`, `awk`.
//!
//! ## Quick example
//!
//! ```rust
//! use polysh::translator::translate_command;
//! use polysh::detector::ShellInfo;
//! use polysh::mappings::Dialect;
//!
//! let shell = ShellInfo {
//!     dialect: Dialect::PowerShell,
//!     supports_conditional_connectors: true,
//!     needs_unix_translation: true,
//!     target: Dialect::PowerShell,
//!     version: Some(7),
//! };
//!
//! let result = translate_command("rm -rf dist && echo done", &shell);
//! // → "Remove-Item -Recurse -Force dist; if ($?) { Write-Host done }"
//! ```
//!
//! ## Modules
//!
//! - `tokenizer` — Shell command tokenizer
//! - `mappings` — Command mapping registry with bidirectional FlagGroup data
//! - `detector` — Shell detection (platform + environment)
//! - `translator` — Direction-agnostic translation engine

pub mod detector;
pub mod mappings;
pub mod tokenizer;
pub mod translator;
