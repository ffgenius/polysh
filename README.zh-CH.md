# polysh

> Bidirectional shell command translator — **Unix ⇄ PowerShell ⇄ CMD**

[![Crates.io](https://img.shields.io/crates/v/polysh)](https://crates.io/crates/polysh)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[English](README.md) | [中文](README.zh-CH.md)

**polysh** 是一个纯 Rust 编写的 Shell 命令翻译库。将一条 Shell 命令在三种方言之间互译：

```
Unix (bash / zsh / fish)   ⇄   PowerShell   ⇄   CMD (命令提示符)
```

零外部依赖，所有映射数据在编译时内嵌。

## 使用场景

你有一个配置文件，里面写了一条 `sh` 命令：

```yaml
# app-config.yml
build: "rm -rf dist && npm run build"
```

你的 CLI 工具需要在 Windows 上执行等价操作。用 polysh：

```rust
use polysh::detector::detect_shell;
use polysh::mappings::{MappingRegistry, Dialect};
use polysh::translator::{detect_input_format, translate_with_registry};

let cmd = "rm -rf dist && npm run build";

let source = detect_input_format(cmd);         // → Dialect::Unix
let shell  = detect_shell();                    // 当前平台
let reg    = MappingRegistry::new();

let translated = translate_with_registry(
    cmd, source, shell.target, &shell, &reg,
);

// Linux:   "rm -rf dist && npm run build"
// Windows: "Remove-Item -Recurse -Force dist; if ($?) { npm run build }"
```

## 快速开始

```toml
[dependencies]
polysh = "0.0.1"
```

### 最简单的用法：自动检测 + 翻译

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

### 显式指定源和目标

```rust
use polysh::mappings::MappingRegistry;
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

### 手动控制方言检测

```rust
use polysh::translator::detect_input_format;

assert_eq!(detect_input_format("rm -rf dist"), Dialect::Unix);
assert_eq!(detect_input_format("Remove-Item -Force"), Dialect::PowerShell);
assert_eq!(detect_input_format("del /s /q dist"), Dialect::Cmd);
```

### 检查命令是否有未翻译的部分

```rust
use polysh::translator::lint_command;

let result = lint_command("rm -rf dist");
assert!(result.unsupported.is_empty());  // ✅ 全部可翻译

let result = lint_command("unknown_cmd --flag");
assert!(!result.unsupported.is_empty()); // ❌ 有无法翻译的段
```

## API 总览

```rust
// 核心翻译
polysh::translator::translate_command(cmd, &shell) -> String
polysh::translator::translate_with_registry(cmd, from, to, &shell, &reg) -> String

// 格式检测
polysh::translator::detect_input_format(cmd) -> Dialect
polysh::translator::lint_command(cmd) -> LintResult

// Shell 检测
polysh::detector::detect_shell() -> ShellInfo

// 分词（如果需要单独使用）
polysh::tokenizer::tokenize_with_pos(cmd) -> Vec<Token>
polysh::tokenizer::tokenize_with_pos_enhanced_and_roles(cmd) -> Vec<RoleToken>

// 注册表（如果需要自定义翻译逻辑）
polysh::mappings::MappingRegistry::new()
polysh::mappings::MappingRegistry::lookup_cmd(dialect, name) -> Option<&CommandMapping>
polysh::mappings::MappingRegistry::translate_flag(from, cmd, flag, to) -> Option<&str>
```

## 支持的翻译方向

所有 6 个方向都有静态映射（命令名 + 标志位），复杂的命令有动态翻译器：

| 方向 | 静态映射 | 动态翻译 | 典型用例 |
|------|:---:|:---:|------|
| Unix → PowerShell | ✅ | 23 个命令 | Linux 配置 → Windows 执行 |
| PowerShell → Unix | ✅ | 20 个命令 | PS 脚本 → Linux 执行 |
| Unix → CMD | ✅ | 6 个命令 | Linux 配置 → CMD 执行 |
| CMD → Unix | ✅ | 7 个命令 | 批处理 → Linux 执行 |
| PowerShell → CMD | ✅ | 6 个命令 | PS → 纯 CMD |
| CMD → PowerShell | ✅ | 7 个命令 | 批处理 → PowerShell |

### 静态映射覆盖的命令（~100 个）

**文件操作**：`rm` `ls` `cp` `mv` `mkdir` `touch` `cat` `rmdir`

**文本处理**：`grep` `echo` `sort` `uniq` `wc` `head` `tail` `awk` `sed` `cut` `tr` `diff` `tee`

**系统管理**：`ps` `kill` `top` `df` `du` `free` `uptime` `whoami` `hostname` `date` `clear` `which` `uname`

**服务管理**：`systemctl` `shutdown` `reboot`

**文件系统**：`find` `chmod` `chown` `chgrp` `ln` `stat` `dirname` `basename` `realpath`

**压缩归档**：`tar` `gzip` `gunzip` `zip` `unzip` `bzip2` `bunzip2`

**网络**：`curl` `wget` `ping` `ssh` `ifconfig` `netstat` `traceroute` `dig` `nslookup` `route`

**包管理/工具**：`apt` `brew` `npm` `pnpm` `yarn` `pip` `cargo` `make` `cmake` `gcc` `g++`

**版本控制/DevOps**：`git` `docker` `kubectl` `terraform` `ansible` `svn` `vagrant`

**数据库**：`mysql` `psql` `conda`

**用户管理**：`sudo` `useradd` `userdel`

### 动态翻译覆盖的命令

| Unix 命令 | 特殊处理 |
|-----------|---------|
| `find -name -delete` | 组装多 cmdlet 管道 |
| `sed 's/old/new/'` | `-replace` 运算符转换 |
| `awk '{print $N}'` | `ForEach-Object` + `Split` |
| `cut -d -f` | `ForEach-Object` + `Split` |
| `tr 'a' 'b'` | `Replace` / `-replace` |
| `systemctl start/stop/enable...` | PS/CMD service 命令 |
| `chmod 755` / `chown` | `icacls` 权限映射（近似） |
| `ln -s` / `mklink` | 参数顺序自动反转 |
| `sudo` / `runas` | 权限提升转换 |

## 模块结构

```
src/
├── lib.rs           # 库入口，模块声明
├── tokenizer.rs     # 命令字符串 → 有类型的 token 序列
├── translator.rs    # 核心翻译引擎：拆分 → 翻译 → 组装
├── detector.rs      # 检测当前 Shell 环境
└── mappings/
    ├── mod.rs       # MappingRegistry：O(1) 双向查找表
    ├── data.rs      # 静态数据：~100 个 CommandMapping
    └── dynamic.rs   # 动态翻译器：处理复杂参数结构的命令
```

完整的数据流：`命令字符串 → tokenize → split_by_connectors → split_by_pipe → translate_segment → 组装`

## 已知局限

- **命令名冲突**：`Get-ChildItem` 同时映射到 `ls` 和 `umask`，`dir` 同时映射到 `ls` 和 `stat`（HashMap 后插入者胜出）
- **`icacls` 权限映射**：Windows ACL 权限模型比 Unix 八进制复杂，转换是近似值
- **PS `ForEach-Object` 逆向**：依赖子字符串匹配脚本块内容，不是 AST 解析
- **CMD `%VAR%` 变量**：tokenizer 不特殊处理，会当作普通文本

## 许可证

[MIT](./LICENSE)
