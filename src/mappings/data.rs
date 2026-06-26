//! Static command mapping data.
//!
//! Every command mapping is a `CommandMapping` with `FlagGroup` triples.
//! One entry covers all 6 translation directions.
//!
//! Data ported from `smartsh`.

use super::{CommandMapping, FlagGroup};

const fn fg(unix: &'static str, ps: &'static str, cmd: &'static str) -> FlagGroup {
    FlagGroup {
        unix,
        powershell: ps,
        cmd,
    }
}
const fn fg2(unix: &'static str, ps: &'static str) -> FlagGroup {
    FlagGroup {
        unix,
        powershell: ps,
        cmd: "",
    }
}
// ============================================================================
// File Operations
// ============================================================================
const RM: CommandMapping = CommandMapping {
    unix: "rm",
    powershell: "Remove-Item",
    cmd: "del",
    flags: &[
        fg("-rf", "-Recurse -Force", "/s /q"),
        fg("-fr", "-Recurse -Force", "/s /q"),
        fg("-r", "-Recurse", "/s"),
        fg("-f", "-Force", "/q"),
    ],
    force_args: true,
};
const MKDIR: CommandMapping = CommandMapping {
    unix: "mkdir",
    powershell: "New-Item -ItemType Directory",
    cmd: "md",
    flags: &[
        fg("-p", "-Force", ""),
        fg("-m", "-Mode", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: true,
};
const LS: CommandMapping = CommandMapping {
    unix: "ls",
    powershell: "Get-ChildItem",
    cmd: "dir",
    flags: &[
        fg("-la", "-Force", "/a"),
        fg("-al", "-Force", "/a"),
        fg("-a", "-Force", "/a"),
        fg("-l", "", ""),
    ],
    force_args: false,
};
const CP: CommandMapping = CommandMapping {
    unix: "cp",
    powershell: "Copy-Item",
    cmd: "copy",
    flags: &[
        fg("-rf", "-Recurse -Force", "/s /y"),
        fg("-fr", "-Recurse -Force", "/s /y"),
        fg("-r", "-Recurse", "/s"),
        fg("-R", "-Recurse", "/s"),
        fg("-f", "-Force", "/y"),
    ],
    force_args: true,
};
const MV: CommandMapping = CommandMapping {
    unix: "mv",
    powershell: "Move-Item",
    cmd: "move",
    flags: &[],
    force_args: true,
};
const TOUCH: CommandMapping = CommandMapping {
    unix: "touch",
    powershell: "New-Item -ItemType File",
    cmd: "type nul >",
    flags: &[
        fg("-a", "-AccessTime", ""),
        fg("-m", "-ModifyTime", ""),
        fg("-c", "-NoCreate", ""),
    ],
    force_args: true,
};
const CAT: CommandMapping = CommandMapping {
    unix: "cat",
    powershell: "Get-Content",
    cmd: "type",
    flags: &[],
    force_args: true,
};
const GREP: CommandMapping = CommandMapping {
    unix: "grep",
    powershell: "Select-String",
    cmd: "findstr",
    flags: &[
        fg("-i", "-CaseSensitive:$false", "/i"),
        fg("-n", "-LineNumber", "/n"),
        fg("-v", "-NotMatch", "/v"),
        fg("-q", "-Quiet", ""),
        fg("-E", "", ""),
        fg("-F", "-SimpleMatch", ""),
    ],
    force_args: true,
};

// ============================================================================
// Navigation & System Info
// ============================================================================
const PWD: CommandMapping = CommandMapping {
    unix: "pwd",
    powershell: "Get-Location",
    cmd: "cd",
    flags: &[],
    force_args: false,
};
const CLEAR: CommandMapping = CommandMapping {
    unix: "clear",
    powershell: "Clear-Host",
    cmd: "cls",
    flags: &[],
    force_args: false,
};
const WHOAMI: CommandMapping = CommandMapping {
    unix: "whoami",
    powershell: "$env:USERNAME",
    cmd: "echo %USERNAME%",
    flags: &[],
    force_args: false,
};
const HOSTNAME: CommandMapping = CommandMapping {
    unix: "hostname",
    powershell: "$env:COMPUTERNAME",
    cmd: "echo %COMPUTERNAME%",
    flags: &[],
    force_args: false,
};
const DATE: CommandMapping = CommandMapping {
    unix: "date",
    powershell: "Get-Date",
    cmd: "date",
    flags: &[
        fg("-u", "-UFormat", ""),
        fg("-R", "-Format", ""),
        fg("-I", "-Format", ""),
    ],
    force_args: false,
};
const WHICH: CommandMapping = CommandMapping {
    unix: "which",
    powershell: "Get-Command",
    cmd: "",
    flags: &[],
    force_args: true,
};

// ============================================================================
// Process Management
// ============================================================================
const PS_CMD: CommandMapping = CommandMapping {
    unix: "ps",
    powershell: "Get-Process",
    cmd: "tasklist",
    flags: &[],
    force_args: false,
};
const KILL: CommandMapping = CommandMapping {
    unix: "kill",
    powershell: "Stop-Process",
    cmd: "taskkill",
    flags: &[fg("-9", "-Force", "/f")],
    force_args: true,
};
const TOP: CommandMapping = CommandMapping {
    unix: "top",
    powershell: "Get-Process | Sort-Object CPU -Descending | Select-Object -First",
    cmd: "tasklist",
    flags: &[],
    force_args: false,
};
const HTOP: CommandMapping = CommandMapping { unix: "htop", powershell: "Get-Process | Sort-Object CPU -Descending | Select-Object -First 20 | Format-Table -AutoSize", cmd: "tasklist", flags: &[], force_args: false };

// ============================================================================
// Text Processing
// ============================================================================
const ECHO: CommandMapping = CommandMapping {
    unix: "echo",
    powershell: "Write-Host",
    cmd: "echo",
    flags: &[],
    force_args: false,
};
const SORT: CommandMapping = CommandMapping {
    unix: "sort",
    powershell: "Sort-Object",
    cmd: "sort",
    flags: &[],
    force_args: false,
};
const UNIQ: CommandMapping = CommandMapping {
    unix: "uniq",
    powershell: "Select-Object -Unique",
    cmd: "uniq",
    flags: &[fg("-c", "-Unique", "")],
    force_args: false,
};
const WC: CommandMapping = CommandMapping {
    unix: "wc",
    powershell: "Measure-Object",
    cmd: "find /c /v",
    flags: &[
        fg("-l", "-Line", "/c"),
        fg("-w", "-Word", ""),
        fg("-c", "-Character", ""),
        fg("-m", "-Character", ""),
    ],
    force_args: false,
};
const HEAD: CommandMapping = CommandMapping {
    unix: "head",
    powershell: "Get-Content | Select-Object -First",
    cmd: "",
    flags: &[fg("-n", "-First", ""), fg("-c", "-TotalCount", "")],
    force_args: true,
};
const TAIL: CommandMapping = CommandMapping {
    unix: "tail",
    powershell: "Get-Content | Select-Object -Last",
    cmd: "",
    flags: &[
        fg("-n", "-Last", ""),
        fg("-c", "-TotalCount", ""),
        fg("-f", "-Wait", ""),
    ],
    force_args: true,
};
const AWK: CommandMapping = CommandMapping {
    unix: "awk",
    powershell: "ForEach-Object",
    cmd: "",
    flags: &[
        fg("-F", "-FieldSeparator", ""),
        fg("-v", "-Variable", ""),
        fg("-f", "-File", ""),
    ],
    force_args: true,
};
const SED: CommandMapping = CommandMapping {
    unix: "sed",
    powershell: "-replace",
    cmd: "",
    flags: &[
        fg("-n", "-NoPrint", ""),
        fg("-e", "-Expression", ""),
        fg("-f", "-File", ""),
        fg("-i", "-InPlace", ""),
    ],
    force_args: true,
};
const CUT: CommandMapping = CommandMapping {
    unix: "cut",
    powershell: "ForEach-Object",
    cmd: "",
    flags: &[
        fg("-d", "-Delimiter", ""),
        fg("-f", "-Fields", ""),
        fg("-c", "-Characters", ""),
    ],
    force_args: true,
};
const TR: CommandMapping = CommandMapping {
    unix: "tr",
    powershell: "ForEach-Object",
    cmd: "",
    flags: &[
        fg("-d", "-Delete", ""),
        fg("-s", "-Squeeze", ""),
        fg("-c", "-Complement", ""),
    ],
    force_args: true,
};

// ============================================================================
// File System
// ============================================================================
const FIND: CommandMapping = CommandMapping {
    unix: "find",
    powershell: "Get-ChildItem -Recurse",
    cmd: "dir /s",
    flags: &[
        fg("-name", "-Filter", ""),
        fg("-type", "", ""),
        fg("-delete", "", ""),
    ],
    force_args: true,
};
const DF: CommandMapping = CommandMapping {
    unix: "df",
    powershell: "Get-PSDrive",
    cmd: "dir",
    flags: &[fg("-h", "", "")],
    force_args: false,
};
const DU: CommandMapping = CommandMapping {
    unix: "du",
    powershell: "Get-ChildItem -Recurse | Measure-Object -Property Length -Sum",
    cmd: "dir /s",
    flags: &[
        fg("-h", "-Recurse", ""),
        fg("-s", "-Recurse", ""),
        fg("-a", "-Recurse", ""),
    ],
    force_args: true,
};
const DIRNAME: CommandMapping = CommandMapping {
    unix: "dirname",
    powershell: "Split-Path -Parent",
    cmd: "cd",
    flags: &[],
    force_args: true,
};
const BASENAME: CommandMapping = CommandMapping {
    unix: "basename",
    powershell: "Split-Path -Leaf",
    cmd: "",
    flags: &[],
    force_args: true,
};
const RMDIR: CommandMapping = CommandMapping {
    unix: "rmdir",
    powershell: "Remove-Item -Directory",
    cmd: "rmdir",
    flags: &[fg("-p", "-Recurse", "/s"), fg("-v", "-Verbose", "")],
    force_args: true,
};
const TEE: CommandMapping = CommandMapping {
    unix: "tee",
    powershell: "Tee-Object -FilePath",
    cmd: "",
    flags: &[fg("-a", "-Append", "")],
    force_args: true,
};
const LN: CommandMapping = CommandMapping {
    unix: "ln",
    powershell: "New-Item",
    cmd: "mklink",
    flags: &[
        fg("-s", "-ItemType SymbolicLink", ""),
        fg("-f", "-Force", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: true,
};
const STAT: CommandMapping = CommandMapping {
    unix: "stat",
    powershell: "Get-Item | Select-Object Name, Length, LastWriteTime, Attributes",
    cmd: "dir",
    flags: &[fg("-f", "-Format", ""), fg("-t", "-Terse", "")],
    force_args: true,
};
const CHMOD: CommandMapping = CommandMapping {
    unix: "chmod",
    powershell: "icacls",
    cmd: "icacls",
    flags: &[fg("-R", "/T", "/T"), fg("-v", "/Q", "")],
    force_args: true,
};
const CHOWN: CommandMapping = CommandMapping {
    unix: "chown",
    powershell: "icacls",
    cmd: "icacls",
    flags: &[fg("-R", "/T", "/T"), fg("-v", "/Q", "")],
    force_args: true,
};
const CHGRP: CommandMapping = CommandMapping {
    unix: "chgrp",
    powershell: "icacls",
    cmd: "icacls",
    flags: &[fg("-R", "/T", "/T"), fg("-v", "/Q", "")],
    force_args: true,
};

// ============================================================================
// Archive & Compression
// ============================================================================
const TAR: CommandMapping = CommandMapping {
    unix: "tar",
    powershell: "tar",
    cmd: "tar",
    flags: &[
        fg("-c", "-c", ""),
        fg("-x", "-x", ""),
        fg("-f", "-f", ""),
        fg("-z", "-z", ""),
        fg("-j", "-j", ""),
        fg("-v", "-v", ""),
    ],
    force_args: true,
};
const GZIP: CommandMapping = CommandMapping {
    unix: "gzip",
    powershell: "Compress-Archive",
    cmd: "",
    flags: &[
        fg("-d", "-DestinationPath", ""),
        fg("-r", "-Recurse", ""),
        fg("-f", "-Force", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: false,
};
const GUNZIP: CommandMapping = CommandMapping {
    unix: "gunzip",
    powershell: "Expand-Archive",
    cmd: "",
    flags: &[
        fg("-f", "-Force", ""),
        fg("-v", "-Verbose", ""),
        fg("-l", "-ListOnly", ""),
    ],
    force_args: false,
};
const ZIP_CMD: CommandMapping = CommandMapping {
    unix: "zip",
    powershell: "Compress-Archive",
    cmd: "",
    flags: &[
        fg("-r", "-Recurse", ""),
        fg("-f", "-Force", ""),
        fg("-u", "-Update", ""),
        fg("-d", "-DestinationPath", ""),
    ],
    force_args: true,
};
const UNZIP: CommandMapping = CommandMapping {
    unix: "unzip",
    powershell: "Expand-Archive",
    cmd: "",
    flags: &[
        fg("-l", "-ListOnly", ""),
        fg("-o", "-Force", ""),
        fg("-d", "-DestinationPath", ""),
        fg("-q", "-Quiet", ""),
    ],
    force_args: true,
};
const BZIP2: CommandMapping = CommandMapping {
    unix: "bzip2",
    powershell: "Compress-Archive -CompressionLevel Optimal",
    cmd: "",
    flags: &[
        fg("-d", "-Decompress", ""),
        fg("-k", "-Keep", ""),
        fg("-f", "-Force", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: true,
};
const BUNZIP2: CommandMapping = CommandMapping {
    unix: "bunzip2",
    powershell: "Expand-Archive",
    cmd: "",
    flags: &[
        fg("-k", "-Keep", ""),
        fg("-f", "-Force", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: true,
};

// ============================================================================
// Network
// ============================================================================
const CURL: CommandMapping = CommandMapping {
    unix: "curl",
    powershell: "Invoke-WebRequest",
    cmd: "curl",
    flags: &[
        fg("-o", "-OutFile", ""),
        fg("-O", "-OutFile", ""),
        fg("-s", "-UseBasicParsing", ""),
        fg("-L", "-MaximumRedirection", ""),
        fg("-H", "-Headers", ""),
        fg("-d", "-Body", ""),
        fg("-X", "-Method", ""),
        fg("-k", "-SkipCertificateCheck", ""),
    ],
    force_args: true,
};
const WGET: CommandMapping = CommandMapping {
    unix: "wget",
    powershell: "Invoke-WebRequest",
    cmd: "",
    flags: &[
        fg("-O", "-OutFile", ""),
        fg("-o", "-OutFile", ""),
        fg("-q", "-UseBasicParsing", ""),
        fg("-c", "-Resume", ""),
    ],
    force_args: true,
};
const PING: CommandMapping = CommandMapping {
    unix: "ping",
    powershell: "Test-Connection",
    cmd: "ping",
    flags: &[
        fg("-c", "-Count", "-n"),
        fg("-i", "-Interval", ""),
        fg("-t", "-TimeoutSeconds", "-w"),
        fg("-W", "-TimeoutSeconds", ""),
        fg("-s", "-BufferSize", ""),
    ],
    force_args: true,
};
const SSH_CMD: CommandMapping = CommandMapping {
    unix: "ssh",
    powershell: "ssh",
    cmd: "ssh",
    flags: &[
        fg("-p", "-Port", "-p"),
        fg("-i", "-IdentityFile", "-i"),
        fg("-X", "-X11Forwarding", ""),
    ],
    force_args: true,
};
const IFCONFIG: CommandMapping = CommandMapping {
    unix: "ifconfig",
    powershell: "Get-NetAdapter | Format-Table Name, Status, LinkSpeed, MacAddress -AutoSize",
    cmd: "ipconfig",
    flags: &[fg("-a", "-All", "/all"), fg("-s", "-Statistics", "")],
    force_args: false,
};
const NETSTAT: CommandMapping = CommandMapping {
    unix: "netstat",
    powershell: "Get-NetTCPConnection",
    cmd: "netstat",
    flags: &[
        fg("-l", "-State Listen", "-a"),
        fg("-a", "-State Listen", "-a"),
        fg("-n", "", "-n"),
        fg("-p", "", "-p"),
    ],
    force_args: false,
};
const TRACEROUTE: CommandMapping = CommandMapping {
    unix: "traceroute",
    powershell: "Test-NetConnection -TraceRoute",
    cmd: "tracert",
    flags: &[
        fg("-n", "-NoResolve", "-d"),
        fg("-w", "-TimeoutSeconds", "-w"),
        fg("-m", "-MaxHops", "-h"),
    ],
    force_args: true,
};
const DIG: CommandMapping = CommandMapping {
    unix: "dig",
    powershell: "Resolve-DnsName",
    cmd: "nslookup",
    flags: &[
        fg("+short", "-Type A", ""),
        fg("+trace", "-Type NS", ""),
        fg("-x", "-Type PTR", ""),
    ],
    force_args: true,
};
const NSLOOKUP: CommandMapping = CommandMapping {
    unix: "nslookup",
    powershell: "Resolve-DnsName",
    cmd: "nslookup",
    flags: &[fg("-type", "-Type", ""), fg("-server", "-Server", "")],
    force_args: true,
};
const LSOF: CommandMapping = CommandMapping {
    unix: "lsof",
    powershell: "Get-NetTCPConnection",
    cmd: "netstat -ano",
    flags: &[fg("-i", "-Internet", ""), fg("-p", "-Process", "")],
    force_args: false,
};
const TELNET: CommandMapping = CommandMapping {
    unix: "telnet",
    powershell: "Test-NetConnection",
    cmd: "telnet",
    flags: &[fg("-p", "-Port", "")],
    force_args: true,
};
const ROUTE: CommandMapping = CommandMapping { unix: "route", powershell: "Get-NetRoute | Format-Table DestinationPrefix, NextHop, RouteMetric, InterfaceAlias -AutoSize", cmd: "route print",
    flags: &[fg("-n","-NoResolve",""),fg("-e","-Extended","")], force_args: false };

// ============================================================================
// Package & Build Tools
// ============================================================================
const APT: CommandMapping = CommandMapping {
    unix: "apt",
    powershell: "winget",
    cmd: "",
    flags: &[
        fg("install", "install", ""),
        fg("remove", "uninstall", ""),
        fg("update", "upgrade", ""),
        fg("upgrade", "upgrade", ""),
        fg("search", "search", ""),
        fg("list", "list", ""),
    ],
    force_args: false,
};
const BREW: CommandMapping = CommandMapping {
    unix: "brew",
    powershell: "winget",
    cmd: "",
    flags: &[
        fg("install", "install", ""),
        fg("uninstall", "uninstall", ""),
        fg("update", "upgrade", ""),
        fg("upgrade", "upgrade", ""),
        fg("search", "search", ""),
        fg("list", "list", ""),
    ],
    force_args: false,
};
const NPM: CommandMapping = CommandMapping {
    unix: "npm",
    powershell: "npm",
    cmd: "npm",
    flags: &[
        fg("install", "install", "install"),
        fg("uninstall", "uninstall", "uninstall"),
        fg("update", "update", "update"),
        fg("run", "run", "run"),
        fg("test", "test", "test"),
        fg("build", "build", "build"),
    ],
    force_args: false,
};
const PNPM: CommandMapping = CommandMapping {
    unix: "pnpm",
    powershell: "pnpm",
    cmd: "pnpm",
    flags: &[
        fg("install", "install", "install"),
        fg("add", "add", "add"),
        fg("remove", "remove", "remove"),
        fg("run", "run", "run"),
        fg("test", "test", "test"),
        fg("build", "build", "build"),
    ],
    force_args: false,
};
const YARN: CommandMapping = CommandMapping {
    unix: "yarn",
    powershell: "yarn",
    cmd: "yarn",
    flags: &[
        fg("install", "install", "install"),
        fg("add", "add", "add"),
        fg("remove", "remove", "remove"),
        fg("run", "run", "run"),
    ],
    force_args: false,
};
const PIP: CommandMapping = CommandMapping {
    unix: "pip",
    powershell: "pip",
    cmd: "pip",
    flags: &[
        fg("install", "install", "install"),
        fg("uninstall", "uninstall", "uninstall"),
        fg("list", "list", "list"),
        fg("freeze", "freeze", "freeze"),
    ],
    force_args: false,
};
const CARGO_CMD: CommandMapping = CommandMapping {
    unix: "cargo",
    powershell: "cargo",
    cmd: "cargo",
    flags: &[
        fg("build", "build", "build"),
        fg("run", "run", "run"),
        fg("test", "test", "test"),
        fg("check", "check", "check"),
        fg("clean", "clean", "clean"),
        fg("update", "update", "update"),
    ],
    force_args: false,
};
const MAKE: CommandMapping = CommandMapping {
    unix: "make",
    powershell: "make",
    cmd: "make",
    flags: &[
        fg("-j", "-Jobs", ""),
        fg("-f", "-File", ""),
        fg("-C", "-Directory", ""),
    ],
    force_args: false,
};
const CMAKE: CommandMapping = CommandMapping {
    unix: "cmake",
    powershell: "cmake",
    cmd: "cmake",
    flags: &[
        fg("build", "--build", ""),
        fg("install", "--install", ""),
        fg("test", "--test", ""),
    ],
    force_args: false,
};
const GCC: CommandMapping = CommandMapping {
    unix: "gcc",
    powershell: "gcc",
    cmd: "gcc",
    flags: &[
        fg("-o", "-Output", "-o"),
        fg("-c", "-Compile", "-c"),
        fg("-g", "-Debug", "-g"),
    ],
    force_args: true,
};
const GIT: CommandMapping = CommandMapping {
    unix: "git",
    powershell: "git",
    cmd: "git",
    flags: &[
        fg("clone", "clone", "clone"),
        fg("pull", "pull", "pull"),
        fg("push", "push", "push"),
        fg("commit", "commit", "commit"),
        fg("add", "add", "add"),
        fg("status", "status", "status"),
        fg("log", "log", "log"),
        fg("branch", "branch", "branch"),
        fg("checkout", "checkout", "checkout"),
    ],
    force_args: false,
};
const DOCKER: CommandMapping = CommandMapping {
    unix: "docker",
    powershell: "docker",
    cmd: "docker",
    flags: &[
        fg("run", "run", "run"),
        fg("build", "build", "build"),
        fg("ps", "ps", "ps"),
        fg("images", "images", "images"),
    ],
    force_args: false,
};
const KUBECTL: CommandMapping = CommandMapping {
    unix: "kubectl",
    powershell: "kubectl",
    cmd: "kubectl",
    flags: &[
        fg("get", "get", "get"),
        fg("apply", "apply", "apply"),
        fg("delete", "delete", "delete"),
        fg("logs", "logs", "logs"),
    ],
    force_args: false,
};
const TERRAFORM: CommandMapping = CommandMapping {
    unix: "terraform",
    powershell: "terraform",
    cmd: "terraform",
    flags: &[
        fg("init", "init", "init"),
        fg("plan", "plan", "plan"),
        fg("apply", "apply", "apply"),
        fg("destroy", "destroy", "destroy"),
    ],
    force_args: false,
};
const ANSIBLE: CommandMapping = CommandMapping {
    unix: "ansible",
    powershell: "ansible",
    cmd: "ansible",
    flags: &[
        fg("-i", "-Inventory", ""),
        fg("-m", "-Module", ""),
        fg("-a", "-Args", ""),
        fg("-v", "-Verbose", ""),
    ],
    force_args: true,
};

// ============================================================================
// System Services
// ============================================================================
const SYSTEMCTL: CommandMapping = CommandMapping {
    unix: "systemctl",
    powershell: "Get-Service",
    cmd: "sc",
    flags: &[
        fg("start", "Start-Service", "start"),
        fg("stop", "Stop-Service", "stop"),
        fg("restart", "Restart-Service", ""),
        fg("status", "Get-Service", "query"),
        fg("enable", "Set-Service -StartupType Automatic", ""),
        fg("disable", "Set-Service -StartupType Disabled", ""),
    ],
    force_args: true,
};
const UPTIME: CommandMapping = CommandMapping {
    unix: "uptime",
    powershell: "(Get-Date) - (Get-CimInstance Win32_OperatingSystem).LastBootUpTime",
    cmd: "",
    flags: &[],
    force_args: false,
};
const FREE: CommandMapping = CommandMapping {
    unix: "free",
    powershell: "Get-Counter '\\Memory\\Available MBytes'",
    cmd: "",
    flags: &[
        fg("-h", "-Human", ""),
        fg("-m", "-MB", ""),
        fg("-g", "-GB", ""),
    ],
    force_args: false,
};
const SHUTDOWN: CommandMapping = CommandMapping {
    unix: "shutdown",
    powershell: "Stop-Computer",
    cmd: "shutdown",
    flags: &[fg("-h", "", "/s"), fg("-r", "", "/r"), fg("-c", "", "/c")],
    force_args: false,
};
const REBOOT: CommandMapping = CommandMapping {
    unix: "reboot",
    powershell: "Restart-Computer",
    cmd: "shutdown /r",
    flags: &[fg("-f", "-Force", "/f")],
    force_args: false,
};

// ============================================================================
// User Management
// ============================================================================
const SUDO: CommandMapping = CommandMapping {
    unix: "sudo",
    powershell: "Start-Process powershell -Verb RunAs -ArgumentList",
    cmd: "runas",
    flags: &[],
    force_args: true,
};
const USERADD: CommandMapping = CommandMapping {
    unix: "useradd",
    powershell: "New-LocalUser",
    cmd: "net user",
    flags: &[fg("-m", "-Name", "/add")],
    force_args: false,
};
const USERDEL: CommandMapping = CommandMapping {
    unix: "userdel",
    powershell: "Remove-LocalUser",
    cmd: "net user",
    flags: &[fg("-r", "-Name", "/delete"), fg("-f", "-Force", "")],
    force_args: true,
};

// ============================================================================
// Unix→PS only (no CMD equivalent)
// ============================================================================
const DIFF: CommandMapping = CommandMapping {
    unix: "diff",
    powershell: "Compare-Object",
    cmd: "",
    flags: &[
        fg2("-u", "-Unified"),
        fg2("-r", "-Recurse"),
        fg2("-i", "-CaseInsensitive"),
        fg2("-w", "-IgnoreWhiteSpace"),
    ],
    force_args: true,
};
const SPLIT: CommandMapping = CommandMapping {
    unix: "split",
    powershell: "Split-Content",
    cmd: "",
    flags: &[
        fg2("-l", "-LineCount"),
        fg2("-b", "-ByteCount"),
        fg2("-n", "-Number"),
    ],
    force_args: true,
};
const PASTE: CommandMapping = CommandMapping {
    unix: "paste",
    powershell: "Join-Object",
    cmd: "",
    flags: &[fg2("-d", "-Delimiter"), fg2("-s", "-Serial")],
    force_args: true,
};
const RSYNC: CommandMapping = CommandMapping {
    unix: "rsync",
    powershell: "Copy-Item",
    cmd: "",
    flags: &[
        fg2("-a", "-Recurse"),
        fg2("-v", "-Verbose"),
        fg2("-r", "-Recurse"),
        fg2("-u", "-Force"),
        fg2("-n", "-WhatIf"),
    ],
    force_args: true,
};
const LESS: CommandMapping = CommandMapping {
    unix: "less",
    powershell: "Get-Content | Out-Host -Paging",
    cmd: "",
    flags: &[fg2("-N", "-LineNumber")],
    force_args: false,
};
const MORE: CommandMapping = CommandMapping {
    unix: "more",
    powershell: "Get-Content | Out-Host -Paging",
    cmd: "",
    flags: &[],
    force_args: false,
};
const JOBS: CommandMapping = CommandMapping {
    unix: "jobs",
    powershell: "Get-Job",
    cmd: "",
    flags: &[],
    force_args: false,
};
const BG: CommandMapping = CommandMapping {
    unix: "bg",
    powershell: "Resume-Job",
    cmd: "",
    flags: &[],
    force_args: false,
};
const FG: CommandMapping = CommandMapping {
    unix: "fg",
    powershell: "Receive-Job",
    cmd: "",
    flags: &[],
    force_args: false,
};
const NICE: CommandMapping = CommandMapping {
    unix: "nice",
    powershell: "Start-Process",
    cmd: "",
    flags: &[fg2("-n", "-Priority")],
    force_args: true,
};
const NOHUP: CommandMapping = CommandMapping {
    unix: "nohup",
    powershell: "Start-Process",
    cmd: "",
    flags: &[],
    force_args: true,
};
const UMASK: CommandMapping = CommandMapping {
    unix: "umask",
    powershell: "Get-ChildItem",
    cmd: "",
    flags: &[],
    force_args: false,
};
const MKTEMP: CommandMapping = CommandMapping {
    unix: "mktemp",
    powershell: "New-TemporaryFile",
    cmd: "",
    flags: &[fg2("-d", "")],
    force_args: false,
};
const REALPATH: CommandMapping = CommandMapping {
    unix: "realpath",
    powershell: "Resolve-Path",
    cmd: "",
    flags: &[fg2("-q", "-Quiet"), fg2("-s", "-Relative")],
    force_args: true,
};
const JOIN_CMD: CommandMapping = CommandMapping {
    unix: "join",
    powershell: "Join-Object",
    cmd: "",
    flags: &[
        fg2("-1", "-JoinProperty"),
        fg2("-2", "-MergeProperty"),
        fg2("-t", "-Delimiter"),
    ],
    force_args: true,
};
const COMM: CommandMapping = CommandMapping {
    unix: "comm",
    powershell: "Compare-Object",
    cmd: "",
    flags: &[
        fg2("-1", "-IncludeEqual"),
        fg2("-2", "-IncludeEqual"),
        fg2("-3", "-IncludeEqual"),
    ],
    force_args: true,
};
const UNAME: CommandMapping = CommandMapping {
    unix: "uname",
    powershell: "Get-ComputerInfo",
    cmd: "",
    flags: &[
        fg2("-a", "-a"),
        fg2("-r", "-r"),
        fg2("-m", "-m"),
        fg2("-n", "-n"),
    ],
    force_args: false,
};
const LOCATE: CommandMapping = CommandMapping {
    unix: "locate",
    powershell: "Get-ChildItem -Recurse | Where-Object {$_.Name -like}",
    cmd: "",
    flags: &[fg2("-i", "-CaseInsensitive"), fg2("-n", "-Limit")],
    force_args: true,
};
const REV: CommandMapping = CommandMapping {
    unix: "rev",
    powershell: "ForEach-Object",
    cmd: "",
    flags: &[],
    force_args: false,
};
const TAC: CommandMapping = CommandMapping {
    unix: "tac",
    powershell: "Get-Content | Sort-Object -Descending",
    cmd: "",
    flags: &[],
    force_args: false,
};
const COLUMN: CommandMapping = CommandMapping {
    unix: "column",
    powershell: "Format-Table -AutoSize",
    cmd: "",
    flags: &[fg2("-t", "-AutoSize")],
    force_args: false,
};
const IOTOP: CommandMapping = CommandMapping {
    unix: "iotop",
    powershell: "Get-Process | Sort-Object IO -Descending | Select-Object -First 20",
    cmd: "",
    flags: &[],
    force_args: false,
};
const NMAP: CommandMapping = CommandMapping {
    unix: "nmap",
    powershell: "Test-NetConnection",
    cmd: "",
    flags: &[fg2("-p", "-Port"), fg2("-v", "-Verbose")],
    force_args: true,
};
const CRON: CommandMapping = CommandMapping {
    unix: "cron",
    powershell: "Register-ScheduledJob",
    cmd: "schtasks",
    flags: &[
        fg("-e", "-Edit", "/create"),
        fg("-l", "-List", "/query"),
        fg("-r", "-Remove", "/delete"),
    ],
    force_args: true,
};
const CRONTAB: CommandMapping = CommandMapping {
    unix: "crontab",
    powershell: "Get-ScheduledJob",
    cmd: "schtasks /query",
    flags: &[
        fg("-e", "-Edit", "/create"),
        fg("-l", "-List", "/query"),
        fg("-r", "-Remove", "/delete"),
    ],
    force_args: true,
};
const DNF: CommandMapping = CommandMapping {
    unix: "dnf",
    powershell: "winget",
    cmd: "",
    flags: &[
        fg2("install", "install"),
        fg2("remove", "uninstall"),
        fg2("update", "upgrade"),
        fg2("search", "search"),
    ],
    force_args: false,
};
const YUM: CommandMapping = CommandMapping {
    unix: "yum",
    powershell: "winget",
    cmd: "",
    flags: &[
        fg2("install", "install"),
        fg2("remove", "uninstall"),
        fg2("update", "upgrade"),
        fg2("search", "search"),
    ],
    force_args: false,
};
const APT_GET: CommandMapping = CommandMapping {
    unix: "apt-get",
    powershell: "winget",
    cmd: "",
    flags: &[
        fg2("install", "install"),
        fg2("remove", "uninstall"),
        fg2("update", "upgrade"),
        fg2("upgrade", "upgrade"),
    ],
    force_args: false,
};
const GPP: CommandMapping = CommandMapping {
    unix: "g++",
    powershell: "g++",
    cmd: "",
    flags: &[
        fg2("-o", "-o"),
        fg2("-c", "-c"),
        fg2("-g", "-g"),
        fg2("-Wall", "-Wall"),
        fg2("-std", "-std"),
    ],
    force_args: true,
};
const PKILL: CommandMapping = CommandMapping {
    unix: "pkill",
    powershell: "Stop-Process",
    cmd: "taskkill",
    flags: &[fg("-f", "-Name", "/f"), fg("-9", "-Force", "")],
    force_args: true,
};
const PGREP: CommandMapping = CommandMapping {
    unix: "pgrep",
    powershell: "Get-Process",
    cmd: "tasklist",
    flags: &[],
    force_args: true,
};
const KILLALL: CommandMapping = CommandMapping {
    unix: "killall",
    powershell: "Stop-Process",
    cmd: "taskkill",
    flags: &[fg("-9", "-Force", "/f")],
    force_args: true,
};
const RENICE: CommandMapping = CommandMapping {
    unix: "renice",
    powershell: "Set-ProcessPriority",
    cmd: "",
    flags: &[fg2("-p", "-Id")],
    force_args: true,
};
const IOSTAT: CommandMapping = CommandMapping {
    unix: "iostat",
    powershell: "Get-Counter",
    cmd: "",
    flags: &[],
    force_args: false,
};
const VMSTAT: CommandMapping = CommandMapping {
    unix: "vmstat",
    powershell: "Get-Counter",
    cmd: "",
    flags: &[],
    force_args: false,
};
const NL: CommandMapping = CommandMapping {
    unix: "nl",
    powershell: "Get-Content | ForEach-Object",
    cmd: "",
    flags: &[],
    force_args: false,
};
const CHROOT: CommandMapping = CommandMapping {
    unix: "chroot",
    powershell: "Set-Location",
    cmd: "",
    flags: &[],
    force_args: true,
};
const DMESG: CommandMapping = CommandMapping {
    unix: "dmesg",
    powershell: "Get-WinEvent -LogName System",
    cmd: "",
    flags: &[],
    force_args: false,
};
const TRACEPATH: CommandMapping = CommandMapping {
    unix: "tracepath",
    powershell: "Test-NetConnection -TraceRoute -InformationLevel Detailed",
    cmd: "",
    flags: &[],
    force_args: true,
};
const MTR: CommandMapping = CommandMapping {
    unix: "mtr",
    powershell: "Test-NetConnection -TraceRoute -InformationLevel Detailed",
    cmd: "",
    flags: &[],
    force_args: true,
};
const JAVAC: CommandMapping = CommandMapping {
    unix: "javac",
    powershell: "javac",
    cmd: "javac",
    flags: &[fg("-d", "-d", "-d"), fg("-cp", "-cp", "-cp")],
    force_args: true,
};
const JAVA: CommandMapping = CommandMapping {
    unix: "java",
    powershell: "java",
    cmd: "java",
    flags: &[fg("-cp", "-cp", "-cp"), fg("-jar", "-jar", "-jar")],
    force_args: true,
};
const GO: CommandMapping = CommandMapping {
    unix: "go",
    powershell: "go",
    cmd: "go",
    flags: &[
        fg("build", "build", "build"),
        fg("run", "run", "run"),
        fg("test", "test", "test"),
        fg("install", "install", "install"),
        fg("get", "get", "get"),
    ],
    force_args: false,
};
const DOTNET: CommandMapping = CommandMapping {
    unix: "dotnet",
    powershell: "dotnet",
    cmd: "dotnet",
    flags: &[
        fg("build", "build", "build"),
        fg("run", "run", "run"),
        fg("test", "test", "test"),
        fg("publish", "publish", "publish"),
        fg("restore", "restore", "restore"),
    ],
    force_args: false,
};
const MYSQL: CommandMapping = CommandMapping {
    unix: "mysql",
    powershell: "mysql",
    cmd: "mysql",
    flags: &[
        fg("-u", "-User", "-u"),
        fg("-p", "-Password", "-p"),
        fg("-h", "-Host", "-h"),
        fg("-P", "-Port", "-P"),
    ],
    force_args: true,
};
const PSQL: CommandMapping = CommandMapping {
    unix: "psql",
    powershell: "psql",
    cmd: "psql",
    flags: &[
        fg("-U", "-User", "-U"),
        fg("-h", "-Host", "-h"),
        fg("-p", "-Port", "-p"),
        fg("-d", "-Database", "-d"),
    ],
    force_args: true,
};
const SVN: CommandMapping = CommandMapping {
    unix: "svn",
    powershell: "svn",
    cmd: "svn",
    flags: &[
        fg("checkout", "checkout", "checkout"),
        fg("update", "update", "update"),
        fg("commit", "commit", "commit"),
        fg("status", "status", "status"),
        fg("log", "log", "log"),
    ],
    force_args: false,
};
const MERCURIAL: CommandMapping = CommandMapping {
    unix: "hg",
    powershell: "hg",
    cmd: "hg",
    flags: &[
        fg("clone", "clone", "clone"),
        fg("pull", "pull", "pull"),
        fg("push", "push", "push"),
        fg("commit", "commit", "commit"),
        fg("status", "status", "status"),
    ],
    force_args: false,
};
const VAGRANT: CommandMapping = CommandMapping {
    unix: "vagrant",
    powershell: "vagrant",
    cmd: "vagrant",
    flags: &[
        fg("up", "up", "up"),
        fg("down", "down", "down"),
        fg("halt", "halt", "halt"),
        fg("destroy", "destroy", "destroy"),
        fg("ssh", "ssh", "ssh"),
        fg("status", "status", "status"),
    ],
    force_args: false,
};
const CHEF: CommandMapping = CommandMapping {
    unix: "chef",
    powershell: "chef",
    cmd: "chef",
    flags: &[
        fg("client", "client", "client"),
        fg("solo", "solo", "solo"),
        fg("apply", "apply", "apply"),
    ],
    force_args: true,
};
const PUPPET: CommandMapping = CommandMapping {
    unix: "puppet",
    powershell: "puppet",
    cmd: "puppet",
    flags: &[
        fg("apply", "apply", "apply"),
        fg("agent", "agent", "agent"),
        fg("master", "master", "master"),
    ],
    force_args: true,
};
const SALT: CommandMapping = CommandMapping {
    unix: "salt",
    powershell: "salt",
    cmd: "salt",
    flags: &[
        fg("minion", "minion", "minion"),
        fg("master", "master", "master"),
        fg("key", "key", "key"),
    ],
    force_args: true,
};
const CONDA: CommandMapping = CommandMapping {
    unix: "conda",
    powershell: "conda",
    cmd: "conda",
    flags: &[
        fg("install", "install", "install"),
        fg("remove", "remove", "remove"),
        fg("list", "list", "list"),
        fg("create", "create", "create"),
        fg("activate", "activate", "activate"),
        fg("deactivate", "deactivate", "deactivate"),
    ],
    force_args: false,
};
const GRADLE: CommandMapping = CommandMapping {
    unix: "gradle",
    powershell: "gradle",
    cmd: "gradle",
    flags: &[
        fg("build", "build", "build"),
        fg("test", "test", "test"),
        fg("run", "run", "run"),
        fg("clean", "clean", "clean"),
    ],
    force_args: false,
};
const MAVEN: CommandMapping = CommandMapping {
    unix: "mvn",
    powershell: "mvn",
    cmd: "mvn",
    flags: &[
        fg("compile", "compile", "compile"),
        fg("test", "test", "test"),
        fg("package", "package", "package"),
        fg("install", "install", "install"),
        fg("clean", "clean", "clean"),
    ],
    force_args: false,
};
const ANT: CommandMapping = CommandMapping {
    unix: "ant",
    powershell: "ant",
    cmd: "ant",
    flags: &[
        fg("build", "build", "build"),
        fg("clean", "clean", "clean"),
        fg("test", "test", "test"),
    ],
    force_args: false,
};
const COMPOSER: CommandMapping = CommandMapping {
    unix: "composer",
    powershell: "composer",
    cmd: "composer",
    flags: &[
        fg("install", "install", "install"),
        fg("update", "update", "update"),
        fg("require", "require", "require"),
        fg("remove", "remove", "remove"),
    ],
    force_args: false,
};
const PACKER: CommandMapping = CommandMapping {
    unix: "packer",
    powershell: "packer",
    cmd: "packer",
    flags: &[
        fg("build", "build", "build"),
        fg("validate", "validate", "validate"),
        fg("inspect", "inspect", "inspect"),
        fg("init", "init", "init"),
    ],
    force_args: true,
};

// ============================================================================
// All mappings
// ============================================================================
pub static ALL_MAPPINGS: &[CommandMapping] = &[
    RM, MKDIR, LS, CP, MV, TOUCH, CAT, GREP, PWD, CLEAR, WHOAMI, HOSTNAME, DATE, WHICH, PS_CMD,
    KILL, TOP, HTOP, ECHO, SORT, UNIQ, WC, HEAD, TAIL, AWK, SED, CUT, TR, FIND, DF, DU, DIRNAME,
    BASENAME, RMDIR, TEE, LN, STAT, CHMOD, CHOWN, CHGRP, TAR, GZIP, GUNZIP, ZIP_CMD, UNZIP, BZIP2,
    BUNZIP2, CURL, WGET, PING, SSH_CMD, IFCONFIG, NETSTAT, TRACEROUTE, DIG, NSLOOKUP, LSOF, TELNET,
    ROUTE, APT, BREW, NPM, PNPM, YARN, PIP, CARGO_CMD, MAKE, CMAKE, GCC, GIT, DOCKER, KUBECTL,
    TERRAFORM, ANSIBLE, SYSTEMCTL, UPTIME, FREE, SHUTDOWN, REBOOT, SUDO, USERADD, USERDEL, DIFF,
    SPLIT, PASTE, RSYNC, LESS, MORE, JOBS, BG, FG, NICE, NOHUP, UMASK, MKTEMP, REALPATH, JOIN_CMD,
    COMM, UNAME, LOCATE, REV, TAC, COLUMN, IOTOP, NMAP, CRON, CRONTAB, DNF, YUM, APT_GET, GPP,
    PKILL, PGREP, KILLALL, RENICE, IOSTAT, VMSTAT, NL, CHROOT, DMESG, TRACEPATH, MTR, JAVAC, JAVA,
    GO, DOTNET, MYSQL, PSQL, SVN, MERCURIAL, VAGRANT, CHEF, PUPPET, SALT, CONDA, GRADLE, MAVEN,
    ANT, COMPOSER, PACKER,
];
