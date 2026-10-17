# 平台支持

本页描述 CLI platform domain 的当前状态。CLI 支持与 CCR UI 路由是不同契约。

## 支持矩阵

| 平台 | 状态 | Profile 目录 | 设置目标 |
|---|---|---|---|
| Claude Code | 已实现 | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json` |
| Codex | 已实现 | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml` |
| Antigravity CLI | 已实现 | `~/.ccr/platforms/gemini/profiles.db` | `~/.gemini/antigravity-cli/settings.json` |
| Factory Droid | 已实现 | `~/.ccr/platforms/droid/profiles.db` | `~/.factory/settings.json` |
| Qwen CLI | Stub / 未实现 | 预留平台目录 | 平台操作返回未支持 |

Antigravity 的持久化 key 仍是 `gemini`，`agy` 与 `antigravity` 是输入别名。

Claude、Codex、Gemini、Grok、Droid 的活目录是 `profiles.db`。`profiles.toml` 是 import、export 和 `profile open` 的交换格式，示例文件仍是模板。把备份 TOML 放回活路径后，新二进制不会读它。回退要停用新二进制，从 `backups/<platform>/` 放回 TOML，并删除 `profiles.db` 及其 `-wal`、`-shm`。迁移之后的切换次数只在库里。Qwen 不预建目录库。

## 当前命令边界

```bash
ccr platform list
ccr platform list --json

ccr claude profile list
ccr codex profile list
ccr current
```

`platform switch`、`current`、`info`、`init` 和 `profile` 仍可被 Clap 解析，用于提供明确迁移错误；它们不是当前推荐执行路径。Claude/Codex 使用显式命令，其他平台状态通过 `platform list`、配置文件和当前实现确认。

## 平台指南

- [Claude Code](./claude)
- [Codex](./codex)
- [Antigravity CLI](./gemini)
- [Factory Droid](./droid)
- [平台命令迁移](./migration)

Qwen 只有预留 key 和部分数据路径，不应描述为可切换 runtime。当前 UI 只有 Claude、Codex 和 Antigravity 的平台工作区；OpenCode 使用独立工具入口。

## 相关页面

- [`platform` 命令](/reference/commands/platform)
- [UI 模块地图](/guide/ui-modules)
- [CLI 工作流](/guide/cli-workflows)
