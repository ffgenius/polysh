//! Tests migrated from src/mappings/mod.rs and src/mappings/dynamic.rs.
//!
//! Covers MappingRegistry building, flag lookup, bidirectional translation,
//! and dynamic command translation.

use polysh::mappings::dynamic::translate_dynamic;
use polysh::mappings::{Dialect, MappingRegistry};

// ============================================================================
// Registry tests (from src/mappings/mod.rs)
// ============================================================================

#[test]
fn test_registry_builds() {
    let reg = MappingRegistry::new();
    // Should have entries
    assert!(reg.command_count() > 0);
}

#[test]
fn test_unix_to_ps_command_lookup() {
    let reg = MappingRegistry::new();
    assert!(reg.is_known(Dialect::Unix, "rm"));
    assert!(reg.is_known(Dialect::PowerShell, "Remove-Item"));
    assert!(reg.is_known(Dialect::Cmd, "del"));
}

#[test]
fn test_unix_to_ps_flag_translation() {
    let reg = MappingRegistry::new();
    let result = reg.translate_flag(Dialect::Unix, "rm", "-rf", Dialect::PowerShell);
    assert_eq!(result, Some("-Recurse -Force"));
}

#[test]
fn test_ps_to_unix_flag_translation() {
    let reg = MappingRegistry::new();
    let result = reg.translate_flag(
        Dialect::PowerShell,
        "Remove-Item",
        "-Recurse -Force",
        Dialect::Unix,
    );
    // Both "-rf" and "-fr" map to the same PS flag, so either is valid
    assert!(result == Some("-rf") || result == Some("-fr"));
}

#[test]
fn test_unix_to_cmd_flag_translation() {
    let reg = MappingRegistry::new();
    let result = reg.translate_flag(Dialect::Unix, "rm", "-rf", Dialect::Cmd);
    assert_eq!(result, Some("/s /q"));
}

#[test]
fn test_cmd_to_unix_flag_translation() {
    let reg = MappingRegistry::new();
    let result = reg.translate_flag(Dialect::Cmd, "del", "/s /q", Dialect::Unix);
    // Both "-rf" and "-fr" map to the same /s /q, so either is valid
    assert!(result == Some("-rf") || result == Some("-fr"));
}

#[test]
fn test_bidirectional_roundtrip() {
    let reg = MappingRegistry::new();
    // Unix → PS → Unix should give back the same canonical flag
    let ps = reg
        .translate_flag(Dialect::Unix, "rm", "-rf", Dialect::PowerShell)
        .unwrap();
    let back = reg
        .translate_flag(Dialect::PowerShell, "Remove-Item", ps, Dialect::Unix)
        .unwrap();
    // Round-trip is valid as long as it's one of the equivalent flags
    assert!(back == "-rf" || back == "-fr");
}

#[test]
fn test_unknown_flag_returns_none() {
    let reg = MappingRegistry::new();
    let result = reg.translate_flag(Dialect::Unix, "rm", "--nonexistent", Dialect::PowerShell);
    assert_eq!(result, None);
}

// ============================================================================
// Dynamic translation tests (from src/mappings/dynamic.rs)
// ============================================================================

#[test]
fn test_dynamic_sleep_translation() {
    let result = translate_dynamic(
        "sleep",
        &[],
        &["30".to_string()],
        Dialect::Unix,
        Dialect::PowerShell,
    );
    assert_eq!(result, Some("Start-Sleep 30".to_string()));
}

#[test]
fn test_dynamic_whoami_translation() {
    let result = translate_dynamic("whoami", &[], &[], Dialect::Unix, Dialect::PowerShell);
    assert_eq!(result, Some("$env:USERNAME".to_string()));
}

#[test]
fn test_dynamic_find_delete_translation() {
    let result = translate_dynamic(
        "find",
        &[
            "-name".to_string(),
            "*.rs".to_string(),
            "-delete".to_string(),
        ],
        &[".".to_string()],
        Dialect::Unix,
        Dialect::PowerShell,
    );
    let res = result.unwrap();
    assert!(res.contains("Get-ChildItem"));
    assert!(res.contains("Remove-Item"));
}

#[test]
fn test_dynamic_sed_substitute_translation() {
    let result = translate_dynamic(
        "sed",
        &[],
        &["'s/foo/bar/'".to_string(), "file.txt".to_string()],
        Dialect::Unix,
        Dialect::PowerShell,
    );
    assert!(result.unwrap().contains("-replace"));
}

#[test]
fn test_dynamic_systemctl_translation() {
    let result = translate_dynamic(
        "systemctl",
        &[],
        &["start".to_string(), "nginx".to_string()],
        Dialect::Unix,
        Dialect::PowerShell,
    );
    assert_eq!(result, Some("Start-Service nginx".to_string()));
}

#[test]
fn test_dynamic_tr_translation() {
    let result = translate_dynamic(
        "tr",
        &[],
        &["'a'".to_string(), "'b'".to_string()],
        Dialect::Unix,
        Dialect::PowerShell,
    );
    assert!(result.unwrap().contains("Replace"));
}

#[test]
fn test_ps_to_unix_fallback() {
    let result = translate_dynamic("Unknown-Cmd", &[], &[], Dialect::PowerShell, Dialect::Unix);
    assert_eq!(result, None);
}

// ============================================================================
// PS → Unix dynamic tests (new)
// ============================================================================

#[test]
fn test_ps_start_service_to_unix() {
    let r = translate_dynamic(
        "Start-Service",
        &[],
        &["nginx".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl start nginx".to_string()));
}

#[test]
fn test_ps_stop_service_to_unix() {
    let r = translate_dynamic(
        "Stop-Service",
        &[],
        &["nginx".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl stop nginx".to_string()));
}

#[test]
fn test_ps_restart_service_to_unix() {
    let r = translate_dynamic(
        "Restart-Service",
        &[],
        &["nginx".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl restart nginx".to_string()));
}

#[test]
fn test_ps_get_service_to_unix() {
    let r = translate_dynamic(
        "Get-Service",
        &[],
        &["nginx".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl status nginx".to_string()));
}

#[test]
fn test_ps_set_service_enable_to_unix() {
    let r = translate_dynamic(
        "Set-Service",
        &[
            "-Name".to_string(),
            "nginx".to_string(),
            "-StartupType".to_string(),
            "Automatic".to_string(),
        ],
        &[],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl enable nginx".to_string()));
}

#[test]
fn test_ps_set_service_disable_to_unix() {
    let r = translate_dynamic(
        "Set-Service",
        &[
            "-Name".to_string(),
            "nginx".to_string(),
            "-StartupType".to_string(),
            "Disabled".to_string(),
        ],
        &[],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl disable nginx".to_string()));
}

#[test]
fn test_ps_new_item_file_to_unix() {
    let r = translate_dynamic(
        "New-Item",
        &["-ItemType".to_string(), "File".to_string()],
        &["f.txt".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("touch f.txt".to_string()));
}

#[test]
fn test_ps_new_item_dir_to_unix() {
    let r = translate_dynamic(
        "New-Item",
        &["-ItemType".to_string(), "Directory".to_string()],
        &["d".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("mkdir d".to_string()));
}

#[test]
fn test_ps_new_item_symlink_to_unix() {
    let r = translate_dynamic(
        "New-Item",
        &[
            "-ItemType".to_string(),
            "SymbolicLink".to_string(),
            "-Target".to_string(),
            "/etc/hosts".to_string(),
            "-Name".to_string(),
            "hosts".to_string(),
        ],
        &[],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("ln -s /etc/hosts hosts".to_string()));
}

#[test]
fn test_ps_set_location_to_unix() {
    let r = translate_dynamic(
        "Set-Location",
        &[],
        &["/tmp".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("cd /tmp".to_string()));
}

#[test]
fn test_ps_get_content_tail_to_unix() {
    let r = translate_dynamic(
        "Get-Content",
        &["-Tail".to_string(), "10".to_string()],
        &["f.txt".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert!(r.unwrap().contains("tail -n 10"));
}

#[test]
fn test_ps_get_content_wait_to_unix() {
    let r = translate_dynamic(
        "Get-Content",
        &["-Wait".to_string()],
        &["f.txt".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("tail -f f.txt".to_string()));
}

#[test]
fn test_ps_stop_process_force_to_unix() {
    let r = translate_dynamic(
        "Stop-Process",
        &[
            "-Name".to_string(),
            "nginx".to_string(),
            "-Force".to_string(),
        ],
        &[],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("pkill -9 nginx".to_string()));
}

#[test]
fn test_ps_get_item_to_unix() {
    let r = translate_dynamic(
        "Get-Item",
        &[],
        &["f.txt".to_string()],
        Dialect::PowerShell,
        Dialect::Unix,
    );
    assert_eq!(r, Some("stat f.txt".to_string()));
}

// ============================================================================
// CMD → Unix dynamic tests (new)
// ============================================================================

#[test]
fn test_cmd_sc_start_to_unix() {
    let r = translate_dynamic(
        "sc",
        &[],
        &["start".to_string(), "nginx".to_string()],
        Dialect::Cmd,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl start nginx".to_string()));
}

#[test]
fn test_cmd_sc_query_to_unix() {
    let r = translate_dynamic(
        "sc",
        &[],
        &["query".to_string(), "nginx".to_string()],
        Dialect::Cmd,
        Dialect::Unix,
    );
    assert_eq!(r, Some("systemctl status nginx".to_string()));
}

#[test]
fn test_cmd_tasklist_to_unix() {
    let r = translate_dynamic("tasklist", &[], &[], Dialect::Cmd, Dialect::Unix);
    assert_eq!(r, Some("ps aux".to_string()));
}

#[test]
fn test_cmd_taskkill_force_to_unix() {
    let r = translate_dynamic(
        "taskkill",
        &["/f".to_string()],
        &["/im".to_string(), "nginx.exe".to_string()],
        Dialect::Cmd,
        Dialect::Unix,
    );
    assert_eq!(r, Some("pkill -9 nginx.exe".to_string()));
}

#[test]
fn test_cmd_mklink_dir_to_unix() {
    let r = translate_dynamic(
        "mklink",
        &["/D".to_string()],
        &["link".to_string(), "target".to_string()],
        Dialect::Cmd,
        Dialect::Unix,
    );
    assert_eq!(r, Some("ln -s target link".to_string()));
}

#[test]
fn test_cmd_runas_to_unix() {
    let r = translate_dynamic(
        "runas",
        &[],
        &["cmd".to_string()],
        Dialect::Cmd,
        Dialect::Unix,
    );
    assert!(r.unwrap().contains("sudo"));
}

// ============================================================================
// Unix → CMD dynamic tests (new)
// ============================================================================

#[test]
fn test_unix_systemctl_to_cmd() {
    let r = translate_dynamic(
        "systemctl",
        &[],
        &["start".to_string(), "nginx".to_string()],
        Dialect::Unix,
        Dialect::Cmd,
    );
    assert_eq!(r, Some("sc start nginx".to_string()));
}

#[test]
fn test_unix_systemctl_status_to_cmd() {
    let r = translate_dynamic(
        "systemctl",
        &[],
        &["status".to_string(), "nginx".to_string()],
        Dialect::Unix,
        Dialect::Cmd,
    );
    assert_eq!(r, Some("sc query nginx".to_string()));
}

#[test]
fn test_unix_ln_to_cmd() {
    let r = translate_dynamic(
        "ln",
        &["-s".to_string()],
        &["target".to_string(), "link".to_string()],
        Dialect::Unix,
        Dialect::Cmd,
    );
    assert_eq!(r, Some("mklink /D link target".to_string()));
}

#[test]
fn test_unix_sudo_to_cmd() {
    let r = translate_dynamic(
        "sudo",
        &[],
        &["ls".to_string()],
        Dialect::Unix,
        Dialect::Cmd,
    );
    assert!(r.unwrap().contains("runas"));
}

// ============================================================================
// PS → CMD dynamic tests (new)
// ============================================================================

#[test]
fn test_ps_start_service_to_cmd() {
    let r = translate_dynamic(
        "Start-Service",
        &[],
        &["nginx".to_string()],
        Dialect::PowerShell,
        Dialect::Cmd,
    );
    assert_eq!(r, Some("sc start nginx".to_string()));
}

#[test]
fn test_ps_set_location_to_cmd() {
    let r = translate_dynamic(
        "Set-Location",
        &[],
        &["C:\\tmp".to_string()],
        Dialect::PowerShell,
        Dialect::Cmd,
    );
    assert_eq!(r, Some("cd /d C:\\tmp".to_string()));
}

// ============================================================================
// CMD → PS dynamic tests (new)
// ============================================================================

#[test]
fn test_cmd_sc_start_to_ps() {
    let r = translate_dynamic(
        "sc",
        &[],
        &["start".to_string(), "nginx".to_string()],
        Dialect::Cmd,
        Dialect::PowerShell,
    );
    assert_eq!(r, Some("Start-Service nginx".to_string()));
}

#[test]
fn test_cmd_tasklist_to_ps() {
    let r = translate_dynamic("tasklist", &[], &[], Dialect::Cmd, Dialect::PowerShell);
    assert_eq!(r, Some("Get-Process".to_string()));
}

#[test]
fn test_cmd_taskkill_to_ps() {
    let r = translate_dynamic(
        "taskkill",
        &["/f".to_string()],
        &["/im".to_string(), "nginx.exe".to_string()],
        Dialect::Cmd,
        Dialect::PowerShell,
    );
    assert_eq!(r, Some("Stop-Process -Name nginx.exe -Force".to_string()));
}

#[test]
fn test_cmd_mklink_dir_to_ps() {
    let r = translate_dynamic(
        "mklink",
        &["/D".to_string()],
        &["link".to_string(), "target".to_string()],
        Dialect::Cmd,
        Dialect::PowerShell,
    );
    assert!(r.unwrap().contains("SymbolicLink"));
}
