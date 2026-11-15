# `platform` - 平台注册表查询与迁移对照

`ccr platform` 仅提供跨平台注册表只读列表与帮助入口，不再承担平台切换、初始化或 Profile 增删改查职责。

## 当前可用子命令

```bash
ccr platform list
ccr platform list --json
ccr platform help
```

- `ccr platform list [--json]`：查看 CCR 可识别的 AI CLI 平台、注册表启用状态及各平台记录的 `current_profile`（支持 `--json` 供脚本与工具消费）。
- `ccr platform help`（或 `ccr platform --help`）：查看 `platform` 子命令帮助。

## 已移除的历史子命令

早期版本中位于 `platform` 下的 `switch`、`current`、`info`、`init` 与 `profile` 子命令已从 Clap 命令解析树中彻底移除。现在调用这些已移除的子命令不再进入自定义迁移提示分支，而是直接由 Clap 返回标准的未知子命令解析错误（退出非零）。

## 完整迁移对照表（含全部 13 个 `profile` 动作）

请按目标平台（`claude`、`codex`、`grok`）改用显式规范命令：

| 已移除的旧 `platform` 子命令形式 | 规范替代命令 |
|---|---|
| 已移除 `platform` 子命令 `list` 之外的注册表查看 `info <platform>` | `ccr platform list [--json]` 或 `ccr <claude\|codex\|grok> profile current [--json]` |
| 已移除 `platform` 子命令 `current` | `ccr current [--verbose \| --json]`（别名 `ccr status` / `ccr show`） |
| 已移除 `platform` 子命令 `switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name> [--json]` 或 `ccr <claude\|codex\|grok> auth switch <name>` |
| 已移除 `platform` 子命令 `init <platform>` | `ccr <claude\|codex\|grok> profile init [--json]` |
| 已移除 `platform` 子命令 `profile open <platform>` | `ccr <claude\|codex\|grok> profile open [--json]` |
| 已移除 `platform` 子命令 `profile current <platform>` | `ccr <claude\|codex\|grok> profile current [--json]` |
| 已移除 `platform` 子命令 `profile list <platform>` | `ccr <claude\|codex\|grok> profile list [--json]` |
| 已移除 `platform` 子命令 `profile switch <platform> <name>` | `ccr <claude\|codex\|grok> profile switch <name> [--json]` |
| 已移除 `platform` 子命令 `profile create <platform> <name> ...` | `ccr <claude\|codex\|grok> profile create <name> [...] [--json]` |
| 已移除 `platform` 子命令 `profile edit <platform> <name> ...` | `ccr <claude\|codex\|grok> profile edit <name> [...] [--json]` |
| 已移除 `platform` 子命令 `profile enable <platform> <name>` | `ccr <claude\|codex\|grok> profile enable <name> [--json]` |
| 已移除 `platform` 子命令 `profile disable <platform> <name>` | `ccr <claude\|codex\|grok> profile disable <name> [--force] [--json]` |
| 已移除 `platform` 子命令 `profile delete <platform> <name>` | `ccr <claude\|codex\|grok> profile delete <name> [--force] [--json]` |
| 已移除 `platform` 子命令 `profile off <platform>` | `ccr <claude\|codex\|grok> profile off [--json]` |
| 已移除 `platform` 子命令 `profile export <platform> ...` | `ccr <claude\|codex\|grok> profile export [NAME]... [...] [--json]` |
| 已移除 `platform` 子命令 `profile import <platform> ...` | `ccr <claude\|codex\|grok> profile import [<FILE>\|-] [...] [--json]` |

## 注册表兼容说明

- 旧版 `~/.ccr/config.toml` 中遗留的 `default_platform` / `current_platform` 字段仍可兼容反序列化，不会导致报错。
- 当前 Auth 与 Profile 路由的唯一权威真相是各平台自身的 `profiles.db` 目录与对应的宿主运行态配置。

## 相关页面

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok)
- [`current`](./current)
- [CLI 工作流](/guide/cli-workflows)
- [迁移指南](/reference/migration)
