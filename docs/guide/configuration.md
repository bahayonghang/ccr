# 配置指南

基于 CCR v3.20.11 的配置模型、存储结构与平台支持。

## 模式与目录

### Unified Mode（默认，多平台）
```
~/.ccr/
|-- config.toml          # 平台注册
|-- platforms/
|   |-- claude/          # profiles.toml + history/ + backups/
|   |-- codex/
|   |-- gemini/
|   |-- qwen/
|   `-- iflow/
|-- history/             # 全局历史
|-- backups/             # 全局备份
`-- logs/                # 日志文件（按天轮转，保留14天）
```
特点：多平台共存、独立历史与备份、与 CCR UI/CLI 同步使用。

### Legacy Mode（兼容单平台）
```
~/.ccs_config.toml       # 单文件配置
~/.claude/settings.json  # Claude settings
```
在命令前设置环境变量开启：`CCR_LEGACY_MODE=1`，保持与 CCS 兼容。

### 模式判定
1. 若设置 `CCR_ROOT` 或存在 `~/.ccr/config.toml` → Unified Mode
2. 否则使用 Legacy `~/.ccs_config.toml`

### 迁移
```bash
ccr migrate --check
ccr migrate               # 迁移到 Unified
ccr migrate --platform claude
```

## 平台支持与命令

支持的平台：Claude、Codex (GitHub Copilot)、Gemini CLI、Qwen、iFlow。

常用命令：
```bash
ccr platform list
ccr platform switch codex
ccr platform current --json
ccr platform info claude
ccr platform init gemini
```

## 配置生命周期

创建与切换：
```bash
ccr add                         # 向导式创建
ccr list
ccr switch <name>               # 或直接 ccr <name>
ccr enable <name> | ccr disable <name> [--force]
```

校验与优化：
```bash
ccr validate
ccr optimize
```

导入/导出与备份：
```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
ccr clean --days 30 --dry-run
```

历史与审计：
```bash
ccr history -l 50
```

## 配置文件片段（示例）
Unified Mode `~/.ccr/platforms/claude/profiles.toml` 中的 profile 结构示例：
```toml
[profiles.default]
name = "anthropic"
api_key = "sk-***"
base_url = "https://api.anthropic.com"
model = "claude-3-5-sonnet"
```
导出文件可选择剥离敏感字段 (`--no-secrets`)，导入支持 Merge/Replace 并自动备份。

## CCR UI 协同
- CLI 与 CCR UI 共享同一配置/历史/备份目录。
- `ccr ui` 会自动检测本地源码或 `~/.ccr/ccr-ui`，不足时从 GitHub 下载。
- WebDAV 同步、导入导出、平台切换等操作可在 UI 中可视化完成。
