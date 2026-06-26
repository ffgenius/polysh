//! Unified mapping registry for bidirectional shell command translation.
//!
//! All command mappings are stored in `data.rs` as static `CommandMapping` slices.
//! At startup, `MappingRegistry::new()` builds lookup indices over these slices
//! for O(1) translation in any direction.

pub mod data;
pub mod dynamic;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Shell dialect — the three command languages we translate between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dialect {
    Unix,
    PowerShell,
    Cmd,
}

impl Dialect {
    /// All supported dialects.
    pub const ALL: &'static [Dialect] = &[Dialect::Unix, Dialect::PowerShell, Dialect::Cmd];

    /// Try to parse a user-provided dialect name.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unix" | "bash" | "sh" | "ash" | "dash" | "zsh" | "fish" | "ksh" | "tcsh" => {
                Some(Dialect::Unix)
            }
            "powershell" | "ps" | "pwsh" => Some(Dialect::PowerShell),
            "cmd" | "dos" | "batch" => Some(Dialect::Cmd),
            _ => None,
        }
    }

    /// Human-readable name for this dialect.
    pub fn name(&self) -> &'static str {
        match self {
            Dialect::Unix => "Unix",
            Dialect::PowerShell => "PowerShell",
            Dialect::Cmd => "CMD",
        }
    }
}

/// A group of flags that mean the same thing across all three shells.
///
/// One `FlagGroup` expresses a single semantic concept (e.g., "recursive+force")
/// in all three dialects. All 6 translation directions derive from this single
/// source of truth — no duplication, no drift.
#[derive(Debug, Clone, Copy)]
pub struct FlagGroup {
    pub unix: &'static str,
    pub powershell: &'static str,
    pub cmd: &'static str,
}

impl FlagGroup {
    /// Get the flag spelling for a specific dialect.
    pub fn get(&self, dialect: Dialect) -> &'static str {
        match dialect {
            Dialect::Unix => self.unix,
            Dialect::PowerShell => self.powershell,
            Dialect::Cmd => self.cmd,
        }
    }
}

/// A single command mapping covering all three shell dialects.
///
/// Each `CommandMapping` stores:
/// - The command name in each dialect
/// - All flag combinations as `FlagGroup` triples
/// - Whether arguments are required
#[derive(Debug, Clone, Copy)]
pub struct CommandMapping {
    pub unix: &'static str,
    pub powershell: &'static str,
    pub cmd: &'static str,
    pub flags: &'static [FlagGroup],
    pub force_args: bool,
}

impl CommandMapping {
    /// Get the command name for a specific dialect.
    pub fn cmd_name(&self, dialect: Dialect) -> &'static str {
        match dialect {
            Dialect::Unix => self.unix,
            Dialect::PowerShell => self.powershell,
            Dialect::Cmd => self.cmd,
        }
    }
}

// ---------------------------------------------------------------------------
// Mapping registry with pre-built indices
// ---------------------------------------------------------------------------

/// Pre-built lookup indices for fast bidirectional translation.
///
/// Build once at startup from `data::ALL_MAPPINGS`, then all translations
/// run in O(1) without scanning.
pub struct MappingRegistry {
    /// (source_dialect, command_name) → &CommandMapping
    cmd_index: HashMap<(Dialect, &'static str), &'static CommandMapping>,

    /// (source_dialect, command_name, flag) → &FlagGroup
    flag_index: HashMap<(Dialect, &'static str, &'static str), &'static FlagGroup>,
}

impl MappingRegistry {
    /// Build the registry from all static command mappings.
    pub fn new() -> Self {
        let mut cmd_index = HashMap::new();
        let mut flag_index = HashMap::new();

        for mapping in data::ALL_MAPPINGS {
            // Index command names in all three dialects
            if !mapping.unix.is_empty() {
                cmd_index.insert((Dialect::Unix, mapping.unix), mapping);
            }
            if !mapping.powershell.is_empty() {
                cmd_index.insert((Dialect::PowerShell, mapping.powershell), mapping);
            }
            if !mapping.cmd.is_empty() {
                cmd_index.insert((Dialect::Cmd, mapping.cmd), mapping);
            }

            // Index flag groups in all three dialects
            for fg in mapping.flags {
                if !fg.unix.is_empty() {
                    flag_index.insert((Dialect::Unix, mapping.unix, fg.unix), fg);
                }
                if !fg.powershell.is_empty() {
                    flag_index.insert((Dialect::PowerShell, mapping.powershell, fg.powershell), fg);
                }
                if !fg.cmd.is_empty() {
                    flag_index.insert((Dialect::Cmd, mapping.cmd, fg.cmd), fg);
                }
            }
        }

        MappingRegistry {
            cmd_index,
            flag_index,
        }
    }

    /// Look up a command mapping by source dialect and command name.
    pub fn lookup_cmd(&self, source: Dialect, cmd_name: &str) -> Option<&'static CommandMapping> {
        self.cmd_index.get(&(source, cmd_name)).copied()
    }

    /// Look up a flag group by source dialect, command name, and flag.
    pub fn lookup_flag(
        &self,
        source: Dialect,
        cmd_name: &str,
        flag: &str,
    ) -> Option<&'static FlagGroup> {
        self.flag_index.get(&(source, cmd_name, flag)).copied()
    }

    /// Translate a single flag from source dialect to target dialect.
    /// Returns `None` if the flag is not known in this direction.
    pub fn translate_flag(
        &self,
        source: Dialect,
        cmd_name: &str,
        flag: &str,
        target: Dialect,
    ) -> Option<&'static str> {
        let fg = self.lookup_flag(source, cmd_name, flag)?;
        let result = fg.get(target);
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Check if a command is known for a given dialect.
    pub fn is_known(&self, source: Dialect, cmd_name: &str) -> bool {
        self.cmd_index.contains_key(&(source, cmd_name))
    }

    /// Get the number of commands in the registry.
    pub fn command_count(&self) -> usize {
        self.cmd_index.len() / 3 // Each mapping is indexed 1-3 times
    }
}

impl Default for MappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}
