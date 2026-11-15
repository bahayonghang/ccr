# 配置模型

本页说明 CCR 7.x 的配置模式、目录结构、SQLite 配置库（`profiles.db`）以及显式 `auth` / `profile` 路由事实源。

## 模式总览

| 模式 | 适用场景 | 主目录 | 说明 |
|------|----------|--------|------|
| Unified Mode | 多平台工作流，默认推荐 | `~/.ccr/` | 平台注册表、`profiles.db` 配置库、历史记录、备份与日志统一管理。 |
| Legacy Mode | 兼容早期单平台单文件流程 | `~/.ccs_config.toml` | 仅当未设置 `CCR_ROOT` 且不存在 `~/.ccr/config.toml` 时回退使用。 |

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
│   │   └── profiles.db
│   ├── codex/
│   │   └── profiles.db
│   ├── grok/
│   │   └── profiles.db
│   ├── gemini/
│   ├── droid/
│   └── qwen/
├── history/
├── backups/
├── logs/
└── ccr-ui/
```

关键存储规则：

- `config.toml`：平台注册表；当前各平台 Profile 路由指针保存在对应平台条目的 `current_profile` 字段中。
- `platforms/<platform>/profiles.db`：Claude、Codex、Grok 的**主活体存储（SQLite 配置库）**。
- **旧版 `profiles.toml` 自动迁移与交换边界**：
  - 升级到 7.x 后，旧版 `platforms/<platform>/profiles.toml` 会在首次访问时自动迁移进 `profiles.db`，原文件备份至 `backups/<platform>/`。
  - 不要把示例或备份 `profiles.toml` 手工复制回活路径——7.x 二进制以 `profiles.db` 为权威事实源，不会重新读取放回原处的 `profiles.toml`。
  - TOML 格式现专用于 `ccr <claude|codex|grok> profile open` 的编辑器临时投影，以及 `ccr <claude|codex|grok> profile export` / `import` 的 `ExchangeV1` 导入导出。
- `history/` 与 `backups/`：脱敏操作审计与自动备份资源。
- `ccr-ui/`：`ccr ui` 使用的前端与桌面运行资产目录。

## Runtime / Profile 事实源

当前 `auth` / `profile` 主路径按平台显式分离：

- `ccr <claude|codex|grok> auth ...`：管理官方登录账号快照与会话状态
- `ccr <claude|codex|grok> profile ...`：管理 `profiles.db` 配置库并将选定路由写入宿主设置文件
- `ccr current`（别名 `ccr status`、`ccr show`）：并列展示当前运行时状态总览

`~/.ccr/config.toml` 中各平台条目的 `current_profile` 是 Profile 路由指针事实源。旧版注册表中可能仍保留 `default_platform` / `current_platform` 字段以维持向后读取兼容，但它们不再参与路由决策。

## 平台状态与宿主路径

| 平台 | CLI 能力层级 | Profile 权威存储 | 宿主设置目标 |
|------|--------------|------------------|--------------|
| Claude Code (`claude`) | 完整 `profile` + `auth` | `~/.ccr/platforms/claude/profiles.db` | `~/.claude/settings.json`、`~/.claude.json` |
| Codex (`codex`) | 完整 `profile` + `auth` | `~/.ccr/platforms/codex/profiles.db` | `~/.codex/config.toml`、`~/.codex/auth.json` |
| Grok (`grok`) | 完整 `profile` + `auth` | `~/.ccr/platforms/grok/profiles.db` | `~/.grok/config.toml`、`~/.grok/user-settings.json` |
| Antigravity CLI (`gemini`) | 保留兼容适配器（`legacy_adapter`） | 无 CLI `profile`/`auth` 表面 | `~/.gemini/antigravity-cli/settings.json`；MCP：`~/.gemini/antigravity-cli/mcp_config.json` |
| Factory Droid (`droid`) | 保留兼容适配器（`legacy_adapter`） | 无 CLI `profile`/`auth` 表面 | `~/.factory/settings.json` |
| Qwen CLI (`qwen`) | 预留 / Stub | 不预建配置库 | 预留平台键与部分数据域支持 |

## 常见操作

```bash
# 初始化配置库
ccr claude profile init
ccr codex profile init
ccr grok profile init

# 在编辑器中打开或导入导出 ExchangeV1 TOML
ccr claude profile open
ccr claude profile export --no-secrets -o claude-profiles.toml
ccr claude profile import claude-profiles.toml --dry-run

# 切换与诊断
ccr claude profile switch <name>
ccr codex profile switch <name>
ccr grok profile switch <name>
ccr current
ccr validate
ccr doctor
```

## 与 CCR UI / VS Code 的关系

- CLI、TUI、`ccr-ui` 与 `ccr-vscode` 共享同一套 `~/.ccr/config.toml` 注册表与各平台 `profiles.db` 配置库。
- `ccr current` 呈现的显式运行时状态也是 UI 与 VS Code 对齐的状态模型。
- `ccr platform list` 保留为平台注册表兼容视图，不再代表全局当前平台切换开关。

## 相关文档

- [CLI 工作流](/guide/cli-workflows)
- [快速开始](/guide/quick-start)
- [平台支持](/reference/platforms/)
