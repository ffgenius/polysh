//! Dynamic translation for commands that can't be handled by static flag mapping alone.
//!
//! These commands require parsing argument structure (e.g., `find -name "*.rs" -delete`,
//! `sed 's/old/new/'`, `awk '{print $1}'`). Each translator maps from one dialect to another.
//!
//! Coverage summary:
//! - Unix → PS:      23 commands  (find, sed, awk, systemctl, chmod, ln, ...)
//! - PS → Unix:       20 commands  (Start/Stop-Service, New-Item, icacls reverse, ...)
//! - CMD → Unix:       7 commands  (sc, icacls, mklink, taskkill, ...)
//! - Unix → CMD:       6 commands  (systemctl→sc, chmod→icacls, ln→mklink, ...)
//! - PS → CMD:         6 commands  (Start-Service→sc, Set-Location→cd, ...)
//! - CMD → PS:         7 commands  (sc→Start-Service, icacls, mklink, ...)

use super::Dialect;

/// Try to dynamically translate a command that has complex argument structure.
///
/// Returns `Some(translated)` if the command can be dynamically translated,
/// `None` if it should fall through to static mapping or be returned as-is.
pub fn translate_dynamic(
    cmd: &str,
    flags: &[String],
    args: &[String],
    from: Dialect,
    to: Dialect,
) -> Option<String> {
    match (from, to) {
        (Dialect::Unix, Dialect::PowerShell) => translate_unix_to_ps(cmd, flags, args),
        (Dialect::PowerShell, Dialect::Unix) => translate_ps_to_unix(cmd, flags, args),
        (Dialect::Cmd, Dialect::Unix) => translate_cmd_to_unix(cmd, flags, args),
        (Dialect::Unix, Dialect::Cmd) => translate_unix_to_cmd(cmd, flags, args),
        (Dialect::PowerShell, Dialect::Cmd) => translate_ps_to_cmd(cmd, flags, args),
        (Dialect::Cmd, Dialect::PowerShell) => translate_cmd_to_ps(cmd, flags, args),
        // Same-dialect pairs are unreachable (translator returns early when source == target)
        _ => None,
    }
}

// ============================================================================
// Unix → PowerShell dynamic translations (23 commands)
// ============================================================================

fn translate_unix_to_ps(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        "find" => Some(translate_find_unix_to_ps(flags, args)),
        "sed" => translate_sed_unix_to_ps(flags, args),
        "awk" => translate_awk_unix_to_ps(flags, args),
        "cut" => translate_cut_unix_to_ps(flags, args),
        "tr" => translate_tr_unix_to_ps(flags, args),
        "head" => translate_head_tail_unix_to_ps("head", flags, args),
        "tail" => translate_head_tail_unix_to_ps("tail", flags, args),
        "wc" => translate_wc_unix_to_ps(flags, args),
        "systemctl" => translate_systemctl_unix_to_ps(args),
        "chmod" => translate_chmod_unix_to_ps(flags, args),
        "chown" => translate_chown_unix_to_ps(args),
        "ln" => translate_ln_unix_to_ps(flags, args),
        "xargs" => translate_xargs_unix_to_ps(flags, args),
        "du" => translate_du_unix_to_ps(flags, args),
        "sleep" => translate_sleep_unix_to_ps(args),
        "whoami" => Some("$env:USERNAME".to_string()),
        "ping" => translate_ping_unix_to_ps(args),
        "less" | "more" => translate_less_unix_to_ps(flags, args),
        "rmdir" => translate_rmdir_unix_to_ps(flags, args),
        "netstat" => translate_netstat_unix_to_ps(flags),
        "gzip" => translate_gzip_unix_to_ps(flags, args),
        "gunzip" => translate_gunzip_unix_to_ps(flags, args),
        "dig" => translate_dig_unix_to_ps(flags, args),
        "sudo" => translate_sudo_unix_to_ps(args),
        "nl" => translate_nl_unix_to_ps(args),
        "uptime" => Some(
            r#"(Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime"#.to_string(),
        ),
        _ => None,
    }
}

// ============================================================================
// PowerShell → Unix dynamic translations (20 commands)
// ============================================================================

fn translate_ps_to_unix(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        // --- Services (systemctl reverse) ---
        "Start-Service" => Some(format!("systemctl start {}", args.join(" "))),
        "Stop-Service" => Some(format!("systemctl stop {}", args.join(" "))),
        "Restart-Service" => Some(format!("systemctl restart {}", args.join(" "))),
        "Get-Service" => {
            if args.is_empty() {
                Some("systemctl list-units --type=service".to_string())
            } else {
                Some(format!("systemctl status {}", args.join(" ")))
            }
        }
        "Set-Service" => {
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())
                .unwrap_or_default();
            let startup = parse_ps_named_param(flags, "-StartupType");
            match startup.as_deref() {
                Some("Automatic") => Some(format!("systemctl enable {}", name)),
                Some("Disabled") => Some(format!("systemctl disable {}", name)),
                _ => None,
            }
        }

        // --- File system ---
        "New-Item" => translate_new_item_ps_to_unix(flags, args),
        "Set-Location" => {
            if args.is_empty() {
                Some("cd".to_string())
            } else {
                Some(format!("cd {}", args.join(" ")))
            }
        }
        "Get-Item" => {
            if args.is_empty() {
                None
            } else {
                Some(format!("stat {}", args.join(" ")))
            }
        }
        "Get-ChildItem" if args.contains(&"-Recurse".to_string()) => {
            let non_flag_args: Vec<&str> = args.iter()
                .filter(|a| !a.starts_with('-'))
                .map(|s| s.as_str())
                .collect();
            Some(format!("find {}", non_flag_args.join(" ")))
        }

        // --- Content / paging ---
        "Get-Content" => translate_get_content_ps_to_unix(flags, args),
        "Out-Host" if flags.contains(&"-Paging".to_string()) => {
            Some(format!("less {}", args.join(" ")))
        }

        // --- Process management ---
        "Stop-Process" if parse_ps_named_param(flags, "-Name").is_some() => {
            let name = parse_ps_named_param(flags, "-Name").unwrap_or_default();
            if flags.contains(&"-Force".to_string()) {
                Some(format!("pkill -9 {}", name))
            } else {
                Some(format!("killall {}", name))
            }
        }

        // --- Text / stream tools (reverse of awk/cut/tr) ---
        "ForEach-Object" => translate_foreach_ps_to_unix(args),
        "Select-String" => Some(format!("grep {}", args.join(" "))),
        "Select-Object" => {
            if flags.contains(&"-Unique".to_string()) {
                Some(format!("uniq {}", args.join(" ")))
            } else {
                Some(format!("head {}", args.join(" ")))
            }
        }
        "Measure-Object" => {
            if flags.contains(&"-Line".to_string()) {
                Some(format!("wc -l {}", args.join(" ")))
            } else if flags.contains(&"-Word".to_string()) {
                Some(format!("wc -w {}", args.join(" ")))
            } else {
                Some(format!("wc {}", args.join(" ")))
            }
        }

        // --- icacls reverse ---
        "icacls" => translate_icacls_ps_to_unix(flags, args),

        _ => None,
    }
}

// ============================================================================
// CMD → Unix dynamic translations (7 commands)
// ============================================================================

fn translate_cmd_to_unix(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        "sc" => translate_sc_cmd_to_unix(args),
        "icacls" => translate_icacls_cmd_to_unix(flags, args),
        "mklink" => translate_mklink_cmd_to_unix(flags, args),
        "runas" => {
            let sub: Vec<&str> = args.iter()
                .filter(|a| !a.starts_with("/user") && !a.starts_with("/u"))
                .map(|s| s.as_str())
                .collect();
            Some(format!("sudo {}", sub.join(" ")))
        }
        "tasklist" => Some("ps aux".to_string()),
        "taskkill" => translate_taskkill_cmd_to_unix(flags, args),
        "schtasks" if flags.iter().any(|f| f == "/query") => Some("crontab -l".to_string()),
        _ => None,
    }
}

// ============================================================================
// Unix → CMD dynamic translations (6 commands)
// ============================================================================

fn translate_unix_to_cmd(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        "systemctl" => translate_systemctl_unix_to_cmd(args),
        "chmod" => translate_chmod_unix_to_cmd(args),
        "chown" => translate_chown_unix_to_cmd(args),
        "ln" => translate_ln_unix_to_cmd(flags, args),
        "sudo" => {
            if args.is_empty() { return None; }
            Some(format!("runas /user:Administrator \"{}\"", args.join(" ")))
        }
        _ => None,
    }
}

// ============================================================================
// PowerShell → CMD dynamic translations (6 commands)
// ============================================================================

fn translate_ps_to_cmd(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        "Start-Service" => Some(format!("sc start {}", args.join(" "))),
        "Stop-Service" => Some(format!("sc stop {}", args.join(" "))),
        "Get-Service" => {
            if args.is_empty() {
                Some("sc query".to_string())
            } else {
                Some(format!("sc query {}", args.join(" ")))
            }
        }
        "Set-Service" => {
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())
                .unwrap_or_default();
            let startup = parse_ps_named_param(flags, "-StartupType");
            match startup.as_deref() {
                Some("Automatic") => Some(format!("sc config {} start=auto", name)),
                Some("Disabled") => Some(format!("sc config {} start=disabled", name)),
                _ => None,
            }
        }
        "Set-Location" => {
            if args.is_empty() {
                Some("cd".to_string())
            } else {
                Some(format!("cd /d {}", args.join(" ")))
            }
        }
        "New-Item" => translate_new_item_ps_to_cmd(flags, args),
        _ => None,
    }
}

// ============================================================================
// CMD → PowerShell dynamic translations (7 commands)
// ============================================================================

fn translate_cmd_to_ps(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    match cmd {
        "sc" => translate_sc_cmd_to_ps(args),
        "icacls" => translate_icacls_cmd_to_ps(flags, args),
        "mklink" => translate_mklink_cmd_to_ps(flags, args),
        "runas" => {
            let sub: Vec<&str> = args.iter()
                .filter(|a| !a.starts_with("/user") && !a.starts_with("/u"))
                .map(|s| s.as_str())
                .collect();
            Some(format!("Start-Process -Verb RunAs -ArgumentList '{}'", sub.join(" ")))
        }
        "tasklist" => Some("Get-Process".to_string()),
        "taskkill" => translate_taskkill_cmd_to_ps(flags, args),
        "schtasks" if flags.iter().any(|f| f == "/query") => Some("Get-ScheduledJob".to_string()),
        _ => None,
    }
}

// ============================================================================
// Individual translator functions — Unix → PS (existing, unchanged)
// ============================================================================

fn translate_find_unix_to_ps(flags: &[String], args: &[String]) -> String {
    let mut path = String::from(".");
    let mut filter: Option<String> = None;
    let mut want_delete = false;

    for arg in args {
        if !arg.starts_with('-') {
            if path == "." {
                path = arg.clone();
            }
        }
    }

    let mut i = 0;
    while i < flags.len() {
        match flags[i].as_str() {
            "-name" if i + 1 < flags.len() => {
                filter = Some(flags.get(i + 1).cloned().unwrap_or_default());
                i += 2;
                continue;
            }
            "-delete" => {
                want_delete = true;
            }
            _ => {}
        }
        i += 1;
    }

    let filter_part = match &filter {
        Some(f) => {
            let unquoted = f.trim_matches(|c| c == '\'' || c == '"');
            format!(" -Filter \"{}\"", unquoted)
        }
        None => String::new(),
    };

    let mut ps = format!("Get-ChildItem {} -Recurse{}", path, filter_part);
    if want_delete {
        ps.push_str(" | Remove-Item");
    }
    ps
}

fn translate_sed_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    if let Some(first_arg) = args.first() {
        let unq = first_arg.trim_matches(|c| c == '\'' || c == '"');
        if unq.starts_with('s') {
            let delim = unq.chars().nth(1)?;
            let rest = &unq[2..];
            let parts: Vec<&str> = rest.split(delim).collect();
            if parts.len() >= 2 {
                let pattern = parts[0];
                let replacement = parts[1];
                let rest_args = &args[1..];
                return Some(format!(
                    "-replace '{pattern}','{replacement}' {}",
                    rest_args.join(" ")
                ));
            }
        }
    }

    if flags.contains(&"-n".to_string()) && args.len() >= 2 {
        let script = args[1].trim_matches(|c| c == '\'' || c == '"');
        if let Some(caps) = script.strip_suffix('p') {
            if let Ok(n) = caps.parse::<usize>() {
                if n >= 1 {
                    let idx = n - 1;
                    let rest = &args[2..];
                    return Some(format!("Select-Object -Index {} {}", idx, rest.join(" ")));
                }
            }
        }
    }

    None
}

fn translate_awk_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    if let Some(script) = args.first() {
        let unq = script.trim_matches(|c| c == '\'' || c == '"');
        if unq.contains("print") {
            if let Some(pos) = unq.find('$') {
                let after = &unq[pos + 1..];
                let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(field) = num_str.parse::<usize>() {
                    if field >= 1 {
                        let idx = field - 1;
                        let rest = &args[1..];
                        return Some(format!(
                            "ForEach-Object {{ $_.Split()[{}] }} {}",
                            idx,
                            rest.join(" ")
                        ));
                    }
                }
            }
        }
    }
    None
}

fn translate_cut_unix_to_ps(flags: &[String], _args: &[String]) -> Option<String> {
    let mut delim = "\t".to_string();
    let mut field_num: Option<usize> = None;

    let mut i = 0;
    while i < flags.len() {
        match flags[i].as_str() {
            "-d" if i + 1 < flags.len() => {
                delim = flags[i + 1].trim_matches(|c| c == '\'' || c == '"').to_string();
                i += 2;
                continue;
            }
            "-f" if i + 1 < flags.len() => {
                if let Ok(n) = flags[i + 1].parse::<usize>() {
                    field_num = Some(n);
                }
                i += 2;
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(field) = field_num {
        if field >= 1 {
            let idx = field - 1;
            return Some(format!(
                "ForEach-Object {{ $_.Split('{}')[{}] }}",
                delim, idx
            ));
        }
    }
    None
}

fn translate_tr_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    if args.len() >= 2 {
        let from = args[0].trim_matches(|c| c == '\'' || c == '"');
        let to = args[1].trim_matches(|c| c == '\'' || c == '"');
        if from.len() == 1 && to.len() == 1 {
            return Some(format!(
                "ForEach-Object {{ $_.Replace('{}','{}') }}",
                from, to
            ));
        }
        if from.len() == to.len() {
            return Some(format!(
                "ForEach-Object {{ $_ -replace '[{}]','{}' }}",
                from, to
            ));
        }
    }
    None
}

fn translate_head_tail_unix_to_ps(cmd: &str, flags: &[String], args: &[String]) -> Option<String> {
    let flag_name = if cmd == "head" { "-First" } else { "-Last" };
    let mut count: Option<usize> = None;

    for f in flags {
        if let Some(num) = parse_number_flag(f) {
            count = Some(num);
            break;
        }
    }

    let count = count?;
    let target_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    Some(format!(
        "Select-Object {} {} {}",
        flag_name, count,
        target_args.join(" ")
    ))
}

fn translate_wc_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    let mode = if flags.contains(&"-l".to_string()) {
        "-Line"
    } else if flags.contains(&"-w".to_string()) {
        "-Word"
    } else if flags.contains(&"-c".to_string()) || flags.contains(&"-m".to_string()) {
        "-Character"
    } else {
        return None;
    };
    Some(format!("Measure-Object {} {}", mode, args.join(" ")))
}

fn translate_systemctl_unix_to_ps(args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let action = &args[0];
    let service = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match action.as_str() {
        "start" => Some(format!("Start-Service {}", service)),
        "stop" => Some(format!("Stop-Service {}", service)),
        "restart" => Some(format!("Restart-Service {}", service)),
        "status" => Some(format!("Get-Service {}", service)),
        "enable" => Some(format!("Set-Service -Name {} -StartupType Automatic", service)),
        "disable" => Some(format!("Set-Service -Name {} -StartupType Disabled", service)),
        "reload" => Some(format!("Restart-Service {}", service)),
        _ => None,
    }
}

fn translate_chmod_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    let mode = args.first()?;
    if mode.len() == 3 || mode.len() == 4 {
        let mode = if mode.len() == 4 { &mode[1..] } else { mode.as_str() };
        if mode.chars().all(|c| c.is_ascii_digit()) {
            let owner = mode.chars().next()?;
            let perm = match owner {
                '7' => "F", '6' => "M", '5' => "M",
                '4' => "R", '3' => "W", '2' => "W", '1' => "X",
                _ => "R",
            };
            let rest = &args[1..];
            return Some(format!("icacls {} /grant Everyone:{}", rest.join(" "), perm));
        }
    }
    None
}

fn translate_chown_unix_to_ps(args: &[String]) -> Option<String> {
    let owner = args.first()?;
    let rest = &args[1..];
    Some(format!("icacls {} /setowner {}", rest.join(" "), owner))
}

fn translate_ln_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    if flags.contains(&"-s".to_string()) {
        let target = args.first()?;
        let name = args.get(1)?;
        Some(format!("New-Item -ItemType SymbolicLink -Target {} -Name {}", target, name))
    } else {
        let target = args.first()?;
        let name = args.get(1)?;
        Some(format!("New-Item -ItemType HardLink -Target {} -Name {}", target, name))
    }
}

fn translate_xargs_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let sub_cmd = &args[0];
    let sub_args: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    Some(format!("ForEach-Object {{ {} {} $_ }}", sub_cmd, sub_args.join(" ")))
}

fn translate_sleep_unix_to_ps(args: &[String]) -> Option<String> {
    let duration = args.first()?;
    if duration.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("Start-Sleep {}", duration))
    } else {
        None
    }
}

fn translate_du_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    if flags.contains(&"-h".to_string()) {
        Some(format!(
            "Get-Item {} | Select-Object Name, @{{Name='Size(MB)';Expression={{[math]::Round($_.Length/1MB,2)}}}}",
            args.join(" ")
        ))
    } else {
        Some(format!(
            "Get-ChildItem -Recurse {} | Measure-Object -Property Length -Sum",
            args.join(" ")
        ))
    }
}

fn translate_ping_unix_to_ps(args: &[String]) -> Option<String> {
    let target = args.first()?;
    Some(format!("Test-Connection -ComputerName {} -Count 4", target))
}

fn translate_less_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    if flags.contains(&"-N".to_string()) {
        Some(format!(
            "Get-Content {} | ForEach-Object {{ $i++; \"$i`t$_\" }} | Out-Host -Paging",
            args.join(" ")
        ))
    } else {
        Some(format!("Get-Content {} | Out-Host -Paging", args.join(" ")))
    }
}

fn translate_rmdir_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    if flags.contains(&"-p".to_string()) {
        Some(format!("Remove-Item -Directory -Recurse {}", args.join(" ")))
    } else {
        Some(format!("Remove-Item -Directory {}", args.join(" ")))
    }
}

fn translate_netstat_unix_to_ps(flags: &[String]) -> Option<String> {
    if flags.contains(&"-l".to_string()) || flags.contains(&"-a".to_string()) {
        Some("Get-NetTCPConnection -State Listen | Format-Table LocalAddress,LocalPort,RemoteAddress,RemotePort,State -AutoSize".to_string())
    } else if flags.contains(&"-n".to_string()) {
        Some("Get-NetTCPConnection | Format-Table LocalAddress,LocalPort,RemoteAddress,RemotePort,State -AutoSize".to_string())
    } else {
        Some("Get-NetTCPConnection | Select-Object -First 20 | Format-Table LocalAddress,LocalPort,RemoteAddress,RemotePort,State -AutoSize".to_string())
    }
}

fn translate_gzip_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    Some(format!("Compress-Archive -Path {} -DestinationPath {}.zip", args.join(" "), args[0]))
}

fn translate_gunzip_unix_to_ps(_flags: &[String], args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    Some(format!("Expand-Archive -Path {} -DestinationPath .", args.join(" ")))
}

fn translate_dig_unix_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    let target = args.first()?;
    if flags.contains(&"+short".to_string()) {
        Some(format!("Resolve-DnsName {} -Type A | Select-Object -ExpandProperty IPAddress", target))
    } else if flags.contains(&"+trace".to_string()) {
        Some(format!("Resolve-DnsName {} -Type NS", target))
    } else if flags.contains(&"-x".to_string()) {
        Some(format!("Resolve-DnsName {} -Type PTR", target))
    } else {
        Some(format!("Resolve-DnsName {}", target))
    }
}

fn translate_sudo_unix_to_ps(args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let sub_cmd = args.join(" ");
    Some(format!(r#"Start-Process powershell -Verb RunAs -ArgumentList "-Command", "{}""#, sub_cmd))
}

fn translate_nl_unix_to_ps(args: &[String]) -> Option<String> {
    if args.is_empty() {
        Some(r#"Get-Content | ForEach-Object { $i++; "$i`t$_" }"#.to_string())
    } else {
        Some(format!(r#"Get-Content {} | ForEach-Object {{ $i++; "$i`t$_" }}"#, args.join(" ")))
    }
}

// ============================================================================
// Individual translators — PS → Unix (new)
// ============================================================================

fn translate_new_item_ps_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    let item_type = parse_ps_named_param(flags, "-ItemType");
    match item_type.as_deref() {
        Some("File") => {
            if args.is_empty() { None }
            else { Some(format!("touch {}", args.join(" "))) }
        }
        Some("Directory") => {
            Some(format!("mkdir {}", args.join(" ")))
        }
        Some("SymbolicLink") => {
            let target = parse_ps_named_param(flags, "-Target")?;
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())?;
            Some(format!("ln -s {} {}", target, name))
        }
        Some("HardLink") => {
            let target = parse_ps_named_param(flags, "-Target")?;
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())?;
            Some(format!("ln {} {}", target, name))
        }
        _ => None,
    }
}

fn translate_get_content_ps_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    // Get-Content -Tail N → tail -n N
    if let Some(tail_val) = parse_ps_named_param(flags, "-Tail") {
        if let Ok(n) = tail_val.parse::<usize>() {
            let rest_args: Vec<&str> = args.iter()
                .filter(|a| !a.starts_with('-'))
                .map(|s| s.as_str())
                .collect();
            return Some(format!("tail -n {} {}", n, rest_args.join(" ")));
        }
    }
    // Get-Content -Wait → tail -f
    if flags.contains(&"-Wait".to_string()) {
        return Some(format!("tail -f {}", args.join(" ")));
    }
    None
}

fn translate_foreach_ps_to_unix(args: &[String]) -> Option<String> {
    // ForEach-Object { $_.Split('delim')[N] } → cut -d'delim' -f(N+1)
    // ForEach-Object { $_.Replace('a','b') } → tr 'a' 'b'
    let script = args.first()?;
    let unquoted = script.trim_matches(|c| c == '{' || c == '}').trim();

    if let Some(rest) = unquoted.strip_prefix("$_.Split(") {
        // Extract delimiter: 'delim' or "delim"
        let delim_end = rest.find(')')?;
        let delim_section = &rest[..delim_end];
        let delim = delim_section.trim_matches(|c| c == '\'' || c == '"' || c == '(');
        // Extract index after )[N]
        let after = &rest[delim_end + 1..];
        if let Some(idx_start) = after.find('[') {
            let idx_end = after[idx_start..].find(']')?;
            let idx_str = &after[idx_start + 1..idx_start + idx_end];
            if let Ok(idx) = idx_str.parse::<usize>() {
                let field = idx + 1;
                let rest_args = &args[1..];
                return Some(format!(
                    "cut -d'{}' -f{} {}",
                    delim, field,
                    rest_args.join(" ")
                ));
            }
        }
    }

    if let Some(rest) = unquoted.strip_prefix("$_.Replace(") {
        let parts: Vec<&str> = rest.split(',').collect();
        if parts.len() >= 2 {
            let from = parts[0].trim_matches(|c| c == '\'' || c == '"' || c == '(');
            let to = parts[1].trim_matches(|c| c == '\'' || c == '"' || c == ')');
            let rest_args = &args[1..];
            return Some(format!("tr '{}' '{}' {}", from, to, rest_args.join(" ")));
        }
    }

    None
}

fn translate_icacls_ps_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    // icacls file /grant Everyone:F → chmod 777 file
    // Look for /grant in flags (CMD-style reclassified) or args
    let has_grant = flags.iter().any(|f| f == "/grant")
        || args.iter().any(|a| a == "/grant");
    let has_setowner = flags.iter().any(|f| f == "/setowner")
        || args.iter().any(|a| a == "/setowner");

    if has_grant {
        // Find the file (first non-flag arg before /grant)
        let file = args.iter()
            .find(|a| !a.starts_with('/') && !a.starts_with('-') && *a != "/grant")
            .cloned()
            .unwrap_or_default();
        // Find the permission string (after /grant, e.g. Everyone:F)
        let perm_str = args.iter()
            .find(|a| a.contains(':'))
            .map(|a| {
                let parts: Vec<&str> = a.split(':').collect();
                parts.last().copied().unwrap_or("F")
            })
            .unwrap_or("F");
        let octal = icacls_perm_to_octal(&format!("{}{}{}", perm_str, perm_str, perm_str));
        Some(format!("chmod {} {}", octal, file))
    } else if has_setowner {
        let file = args.iter()
            .find(|a| !a.starts_with('/') && !a.starts_with('-') && *a != "/setowner")
            .cloned()
            .unwrap_or_default();
        let owner = args.iter()
            .find(|a| *a != &file && !a.starts_with('/') && *a != "/setowner")
            .cloned()
            .unwrap_or_default();
        Some(format!("chown {} {}", owner, file))
    } else {
        None
    }
}

// ============================================================================
// Individual translators — CMD → Unix (new)
// ============================================================================

fn translate_sc_cmd_to_unix(args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let action = &args[0];
    let service = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match action.as_str() {
        "start" => Some(format!("systemctl start {}", service)),
        "stop" => Some(format!("systemctl stop {}", service)),
        "query" => Some(format!("systemctl status {}", service)),
        _ => None,
    }
}

fn translate_icacls_cmd_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    translate_icacls_ps_to_unix(flags, args) // Same logic
}

fn translate_mklink_cmd_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    // mklink /D link target → ln -s target link  (args reversed!)
    // mklink link target → ln target link        (hard link)
    let is_dir = flags.iter().any(|f| f == "/D" || f == "/d");
    let non_flag: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('/'))
        .map(|s| s.as_str())
        .collect();
    if non_flag.len() < 2 { return None; }
    let link_name = non_flag[0];
    let target = non_flag[1];
    if is_dir {
        Some(format!("ln -s {} {}", target, link_name))
    } else {
        Some(format!("ln {} {}", target, link_name))
    }
}

fn translate_taskkill_cmd_to_unix(flags: &[String], args: &[String]) -> Option<String> {
    let has_force = flags.iter().any(|f| f == "/f" || f == "/F");
    // /im procname → killall/pkill procname
    // /pid N → kill N
    if let Some(pid) = parse_cmd_named_param(flags, args, "/pid") {
        if has_force {
            return Some(format!("kill -9 {}", pid));
        } else {
            return Some(format!("kill {}", pid));
        }
    }
    if let Some(im) = parse_cmd_named_param(flags, args, "/im") {
        if has_force {
            return Some(format!("pkill -9 {}", im));
        } else {
            return Some(format!("killall {}", im));
        }
    }
    None
}

// ============================================================================
// Individual translators — Unix → CMD (new)
// ============================================================================

fn translate_systemctl_unix_to_cmd(args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let action = &args[0];
    let service = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match action.as_str() {
        "start" => Some(format!("sc start {}", service)),
        "stop" => Some(format!("sc stop {}", service)),
        "status" => Some(format!("sc query {}", service)),
        "enable" => Some(format!("sc config {} start=auto", service)),
        "disable" => Some(format!("sc config {} start=disabled", service)),
        _ => None,
    }
}

fn translate_chmod_unix_to_cmd(args: &[String]) -> Option<String> {
    let mode = args.first()?;
    if mode.len() == 3 || mode.len() == 4 {
        let mode = if mode.len() == 4 { &mode[1..] } else { mode.as_str() };
        if mode.chars().all(|c| c.is_ascii_digit()) {
            let owner = mode.chars().next()?;
            let perm = match owner {
                '7' => "F", '6' => "M", '5' => "M",
                '4' => "R", '3' => "W", '2' => "W", '1' => "X",
                _ => "R",
            };
            let rest = &args[1..];
            return Some(format!("icacls {} /grant Everyone:{}", rest.join(" "), perm));
        }
    }
    None
}

fn translate_chown_unix_to_cmd(args: &[String]) -> Option<String> {
    let owner = args.first()?;
    let rest = &args[1..];
    Some(format!("icacls {} /setowner {}", rest.join(" "), owner))
}

fn translate_ln_unix_to_cmd(flags: &[String], args: &[String]) -> Option<String> {
    // ln -s target link → mklink /D link target  (args reversed!)
    if args.len() < 2 { return None; }
    let target = &args[0];
    let link_name = &args[1];
    if flags.contains(&"-s".to_string()) {
        Some(format!("mklink /D {} {}", link_name, target))
    } else {
        Some(format!("mklink {} {}", link_name, target))
    }
}

// ============================================================================
// Individual translators — PS → CMD (new)
// ============================================================================

fn translate_new_item_ps_to_cmd(flags: &[String], args: &[String]) -> Option<String> {
    let item_type = parse_ps_named_param(flags, "-ItemType");
    match item_type.as_deref() {
        Some("File") => {
            if args.is_empty() { None }
            else { Some(format!("type nul > {}", args.join(" "))) }
        }
        Some("Directory") => {
            Some(format!("md {}", args.join(" ")))
        }
        Some("SymbolicLink") => {
            let target = parse_ps_named_param(flags, "-Target")?;
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())?;
            Some(format!("mklink /D {} {}", name, target))
        }
        Some("HardLink") => {
            let target = parse_ps_named_param(flags, "-Target")?;
            let name = parse_ps_named_param(flags, "-Name")
                .or_else(|| args.first().cloned())?;
            Some(format!("mklink {} {}", name, target))
        }
        _ => None,
    }
}

// ============================================================================
// Individual translators — CMD → PS (new)
// ============================================================================

fn translate_sc_cmd_to_ps(args: &[String]) -> Option<String> {
    if args.is_empty() { return None; }
    let action = &args[0];
    let service = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match action.as_str() {
        "start" => Some(format!("Start-Service {}", service)),
        "stop" => Some(format!("Stop-Service {}", service)),
        "query" => Some(format!("Get-Service {}", service)),
        _ => None,
    }
}

fn translate_icacls_cmd_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    // icacls /grant → approximate with Set-Acl or just pass through to PS icacls
    // Since icacls also exists in PS, return the original as a valid PS command
    let has_grant = flags.iter().any(|f| f == "/grant")
        || args.iter().any(|a| a == "/grant");
    if has_grant {
        let file = args.iter()
            .find(|a| !a.starts_with('/') && *a != "/grant")
            .cloned()
            .unwrap_or_default();
        let perm_str = args.iter()
            .find(|a| a.contains(':'))
            .map(|a| {
                let parts: Vec<&str> = a.split(':').collect();
                parts.last().copied().unwrap_or("F")
            })
            .unwrap_or("F");
        let _octal = icacls_perm_to_octal(&format!("{}{}{}", perm_str, perm_str, perm_str));
        return Some(format!("icacls {} /grant Everyone:{}", file, perm_str));
    }
    None
}

fn translate_mklink_cmd_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    let is_dir = flags.iter().any(|f| f == "/D" || f == "/d");
    let non_flag: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('/'))
        .map(|s| s.as_str())
        .collect();
    if non_flag.len() < 2 { return None; }
    let link_name = non_flag[0];
    let target = non_flag[1];
    if is_dir {
        Some(format!("New-Item -ItemType SymbolicLink -Target {} -Name {}", target, link_name))
    } else {
        Some(format!("New-Item -ItemType HardLink -Target {} -Name {}", target, link_name))
    }
}

fn translate_taskkill_cmd_to_ps(flags: &[String], args: &[String]) -> Option<String> {
    let has_force = flags.iter().any(|f| f == "/f" || f == "/F");
    if let Some(im) = parse_cmd_named_param(flags, args, "/im") {
        if has_force {
            return Some(format!("Stop-Process -Name {} -Force", im));
        } else {
            return Some(format!("Stop-Process -Name {}", im));
        }
    }
    if let Some(pid) = parse_cmd_named_param(flags, args, "/pid") {
        return Some(format!("Stop-Process -Id {}", pid));
    }
    None
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse a number from flags like "-10", "-n10", "-n 10", "-c10"
fn parse_number_flag(flag: &str) -> Option<usize> {
    if flag.starts_with('-') && flag.len() > 1 {
        if let Ok(n) = flag[1..].parse::<usize>() {
            return Some(n);
        }
        if flag.len() > 2 && flag.as_bytes()[1].is_ascii_alphabetic() {
            if let Ok(n) = flag[2..].parse::<usize>() {
                return Some(n);
            }
        }
    }
    None
}

/// Parse a PowerShell-style named parameter value from the flags list.
/// E.g., find the value after "-Name" or "-ItemType".
fn parse_ps_named_param(flags: &[String], param_name: &str) -> Option<String> {
    let mut i = 0;
    while i < flags.len() {
        if flags[i] == param_name && i + 1 < flags.len() {
            return Some(flags[i + 1].clone());
        }
        i += 1;
    }
    None
}

/// Parse a CMD-style named parameter value from flags or args.
/// E.g., find value after "/pid" or "/im".
fn parse_cmd_named_param(flags: &[String], args: &[String], param_name: &str) -> Option<String> {
    let param_lower = param_name.to_lowercase();
    // Check flags first
    let mut i = 0;
    while i < flags.len() {
        if flags[i].to_lowercase() == param_lower && i + 1 < flags.len() {
            return Some(flags[i + 1].clone());
        }
        i += 1;
    }
    // Check args
    let mut j = 0;
    while j < args.len() {
        if args[j].to_lowercase() == param_lower && j + 1 < args.len() {
            return Some(args[j + 1].clone());
        }
        j += 1;
    }
    None
}

/// Map an icacls permission character to an octal digit (best-effort approximation).
/// icacls permissions are richer than Unix octal; this is a lossy mapping.
fn icacls_perm_to_octal(perm: &str) -> String {
    let digit = match perm {
        "F" => '7',
        "M" => '6',
        "RX" => '5',
        "R" => '4',
        "W" => '3',
        "X" => '1',
        _ => '4',
    };
    format!("{}{}{}", digit, digit, digit)
}
