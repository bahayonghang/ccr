# 平台支持

本页描述 CCR CLI 平台域的当前能力矩阵。CLI 平台状态与 CCR UI 路由注册是相互独立的契约。

## 支持矩阵

| 平台 | 状态与能力层级 | Profile 权威存储 | 宿主设置目标 |
|---|---|---|---|
| [Claude Code](./claude) (`claude`) | 已实现（完整 CLI/TUI/UI `profile` + `auth`） | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`、`~/.claude.json` |
| [Codex](./codex) (`codex`) | 已实现（完整 CLI/TUI/UI `profile` + `auth`） | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`、`~/.codex/auth.json` |
| [Grok](/reference/commands/grok) (`grok`) | 已实现（完整 CLI/TUI/UI `profile` + `auth`） | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`、`~/.grok/user-settings.json` |
| [Antigravity CLI](./gemini) (`gemini`) | 保留兼容适配器（`legacy_adapter`，无 CLI/TUI `profile` 或 `auth` 子命令） | 无 CLI 配置库切换表面 | `~/.gemini/antigravity-cli/settings.json` |
| [Factory Droid](./droid) (`droid`) | 保留兼容适配器（`legacy_adapter`，无 CLI/TUI `profile` 或 `auth` 子命令） | 无 CLI 配置库切换表面 | `~/.factory/settings.json` |
| Qwen CLI (`qwen`) | Stub / 未实现 | 预留平台键，不预建配置库 | 平台操作返回未支持 |

- Antigravity CLI 的持久化平台键保持为 `gemini`，同时识别输入别名 `gemini-cli`、`antigravity`、`antigravity-cli` 与 `agy`。
- Factory Droid 的持久化平台键为 `droid`，识别输入别名 `droid` 与 `factory`。
- Claude、Codex、Grok 的活体配置库是 SQLite `profiles.db`（旧版 `profiles.toml` 在首次访问时自动迁移并备份至 `backups/<platform>/`）。`profiles.toml` 现作为 `profile export` / `import` 的 `ExchangeV1` 交换格式以及 `profile open` 的编辑器投影使用；手工把备份 TOML 放回活路径不会被 7.x 二进制读取。

## 当前命令边界

```bash
# 查看平台注册表
ccr platform list
ccr platform list --json

# 三大一等平台的显式 Profile 管理
ccr claude profile list
ccr codex profile list
ccr grok profile list

# 跨平台运行时总览与统一体检
ccr current
ccr doctor --all-platforms
```

`ccr platform` 子命令树当前仅保留 `list` 与 `help`；所有 `profile` 与 `auth` 路由操作均通过 `ccr <claude|codex|grok> profile ...` 与 `ccr <claude|codex|grok> auth ...` 显式执行。旧版平台子命令的替代方式见 [平台命令迁移](./migration)。

## 平台指南与命令参考

- [Claude Code 平台指南](./claude)（命令参考：[`claude`](/reference/commands/claude)）
- [Codex 平台指南](./codex)（命令参考：[`codex`](/reference/commands/codex)）
- [Grok 命令与配置参考](/reference/commands/grok)
- [Antigravity CLI 平台说明](./gemini)
- [Factory Droid 平台说明](./droid)
- [平台命令迁移](./migration)

Qwen 仅有预留平台键与部分周边数据路径，不应视为可切换运行时。当前 CCR UI 为 Claude、Codex、Antigravity 提供平台工作区（Grok 支持 CLI/TUI 与桌面命令集成，OpenCode 使用独立工作区入口）。

## 相关页面

- [`platform` 命令](/reference/commands/platform)
- [UI 模块地图](/guide/ui-modules)
- [CLI 工作流](/guide/cli-workflows)
