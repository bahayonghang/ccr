# 配置模型

本页说明 CCR 当前的配置模式、目录结构，以及新的 runtime/profile 路由事实源。

## 模式总览

| 模式 | 适用场景 | 主目录 | 说明 |
|------|----------|--------|------|
| Unified Mode | 多平台工作流，默认推荐 | `~/.ccr/` | 平台注册表、profiles、历史、备份、日志按平台组织。 |
| Legacy Mode | 兼容旧 CCS / 单平台 Claude 流程 | `~/.ccs_config.toml` | 保留旧单文件路径。 |

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
│   └── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

关键说明：

- `config.toml`：平台注册表；当前运行时的事实源是各平台自己的 `current_profile`。
- `platforms/<name>/profiles.toml`：该平台的 profile 集合。
- `history/` / `backups/`：全局审计与回滚资源。
- `ccr-ui/`：`ccr ui` 使用的前端目录。

## Runtime/Profile 事实源

当前 auth/profile 主路径已经迁移为：

- `ccr claude auth ...` / `ccr codex auth ...`：official auth 账号面
- `ccr claude profile ...` / `ccr codex profile ...`：runtime/profile 路由面
- `ccr current`：并列展示 Claude Runtime 与 Codex Runtime

`~/.ccr/config.toml` 中每个平台条目的 `current_profile` 才是 profile 路由事实源。

旧 registry 中可能仍保留 `default_platform` / `current_platform`，但它们不再是 auth/profile 路由真相。

## 平台状态

| 平台 | 状态 | Profile 文件 | 设置目标 |
|------|------|-------------|----------|
| Claude | 已实现 | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` |
| Codex | 已实现 | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` |
| Gemini | 已实现 | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` |
| Droid | 已实现 | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` |
| Qwen | 预留 / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` |

## 常见操作

```bash
ccr init
ccr current
ccr platform list
ccr add
ccr claude profile switch <name>
ccr codex profile switch <name>
ccr validate
ccr doctor
```

## 与 CCR UI / VS Code 的关系

- CLI、`ccr-ui`、`ccr-vscode` 共享同一套 `~/.ccr/` registry 与 profile 文件。
- `ccr current` 的双 runtime 模型也是 UI / VS Code 应该对齐的展示模型。
- `ccr platform list` 仍保留为兼容的注册表视图，不再代表一个全局 active-platform 路由开关。

## 相关文档

- [CLI 工作流](/guide/cli-workflows)
- [快速开始](/guide/quick-start)
- [平台支持](/reference/platforms/)
