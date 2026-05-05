# 配置模型

本页说明 CCR 当前的配置模式、目录结构、平台状态与共享资源位置。

## 模式总览

| 模式 | 适用场景 | 主目录 | 说明 |
|------|----------|--------|------|
| Unified Mode | 多平台、推荐默认模式 | `~/.ccr/` | 平台注册、profiles、历史、备份、日志全部按平台组织。 |
| Legacy Mode | 兼容旧 CCS / 单平台 Claude 工作流 | `~/.ccs_config.toml` | 保留单文件配置路径。 |

模式判定顺序：

1. 设置了 `CCR_ROOT`
2. 存在 `~/.ccr/config.toml`
3. 否则回退到 Legacy Mode

## Unified Mode 目录结构

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── droid/
│   ├── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

关键说明：

- `config.toml`：平台注册表与当前平台指针。
- `platforms/<name>/profiles.toml`：该平台的 profile 集合。
- `history/` 与 `backups/`：全局记录与回滚资源。
- `ccr-ui/`：`ccr ui` 下载或缓存的前端项目目录。

## 平台状态

| 平台 | 状态 | 配置文件 | 设置目标 |
|------|------|----------|----------|
| Claude | 已实现 | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` |
| Codex | 已实现 | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` |
| Gemini | 已实现 | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` |
| Droid | 已实现 | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` |
| Qwen | 预留 / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` |
> `Qwen` 当前仍作为预留 / Stub 保留，核心平台实现尚未完整支持。

## 常见生命周期

### 初始化与切换

```bash
ccr init
ccr platform list
ccr platform switch claude
```

### Profile 管理

```bash
ccr add
ccr list
ccr switch <name>
ccr enable <name>
ccr disable <name> --force
```

### 校验、历史与清理

```bash
ccr validate
ccr history --limit 20
ccr optimize
ccr clean backups --days 30 --dry-run
```

### 导入、导出与恢复

```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
```

## 临时覆盖与即时写入

CCR 区分三类写入：

- `ccr switch`：从 profile 读取并写入目标 settings。
- `ccr temp`：不依赖现有 profile，交互式写入当前 settings。
- `ccr temp-token`：对当前 settings 做临时 token / base_url / model 覆盖。

这些命令都不会改变“命令定义层”的默认值，但会影响当前活动 settings 文件。

## 与 CCR UI 的关系

- CLI 与 `ccr-ui` 共享同一套 profiles、历史、备份和日志目录。
- `ccr ui` 负责把浏览器入口连到这套共享状态。
- UI 页面多于 CLI 命令面，但其事实源仍应回到同一配置模型。

## 相关文档

- [CLI 工作流](/guide/cli-workflows)
- [UI 概览](/guide/ui-overview)
- [平台支持](/reference/platforms/)
