# 平台支持

本页统一描述 CCR 当前的平台状态，避免首页、命令页和平台页各说各话。

## 支持矩阵

| 平台 | 状态 | 配置文件 | 设置目标 | 备注 |
|------|------|----------|----------|------|
| Claude Code | 已实现 | `~/.ccr/platforms/claude/profiles.toml` | `~/.claude/settings.json` | 默认主线平台 |
| Codex | 已实现 | `~/.ccr/platforms/codex/profiles.toml` | `~/.codex/config.toml` | 同时支持 `ccr codex auth` |
| Gemini CLI | 已实现 | `~/.ccr/platforms/gemini/profiles.toml` | `~/.ccr/platforms/gemini/settings.json` | Unified Mode 管理 |
| Factory Droid | 已实现 | `~/.ccr/platforms/droid/profiles.toml` | `~/.factory/settings.json` | 独立 settings 结构 |
| Qwen CLI | 预留 / Stub | `~/.ccr/platforms/qwen/profiles.toml` | `~/.ccr/platforms/qwen/settings.json` | 当前核心实现返回未支持 |

> 平台状态以 `Platform::is_implemented()` 和对应平台实现为准。UI 中有入口，不代表 CLI 平台已经完整可用。

## 快速命令

```bash
ccr platform list
ccr platform switch claude
ccr platform info droid
ccr platform init gemini
```

## 已实现平台

- [Claude Code](./claude)
- [Codex](./codex)
- [Gemini CLI](./gemini)
- [Factory Droid](./droid)

## 预留平台

- `qwen`

它当前的意义是：

- 在 Unified Mode 中预留命名空间
- 在 UI 中预留页面入口
- 为后续实现保留文档位置

## 相关文档

- [platform 命令](/reference/commands/platform)
- [CLI 工作流](/guide/cli-workflows)
- [迁移指南](/reference/migration)
