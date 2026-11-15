# 迁移指南

本页汇总 CCR 7.x 的三项核心迁移说明：平台级显式命令面迁移、`profiles.toml -> profiles.db` 存储模型升级，以及 `ccr-ui` 用量统计向外部 `llmusage` 运行时的迁移。

> CCR 不提供 `ccr migrate` 命令；配置库迁移在首次运行规范命令时自动完成。

## 命令迁移速查表（7.x）

| 旧命令（已移除 / 已退休 / 7.x 隐藏） | 当前规范命令 | 说明 |
|---|---|---|
| `ccr switch <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | 已退休；目标平台在命令路径中显式指定 |
| `ccr <name>` | `ccr <claude\|codex\|grok> profile switch <name>` | 快捷入口已退休 |
| `ccr platform switch <platform>` | `ccr <claude\|codex\|grok> profile switch <name>` / `auth ...` | 子命令已从 Clap 移除；不再依赖全局当前平台 |
| `ccr platform current` | `ccr current`（别名 `ccr status`、`ccr show`） | 查看跨平台运行时状态总览 |
| `ccr platform info <platform>` | `ccr platform list [--json]` / `ccr doctor --platform <name>` | 子命令已移除；改查注册表或体检报告 |
| `ccr platform init <platform>` | `ccr <claude\|codex\|grok> profile init` | 子命令已移除；从内置模板初始化 `profiles.db` |
| `ccr platform profile ...` | `ccr <claude\|codex\|grok> profile <动作>` | 子命令已移除；统一使用三平台 `profile` 命令组 |
| `ccr init` | `ccr <claude\|codex\|grok> profile init`（仓库工作流用 `ccr project init`） | 7.x 隐藏并弃用（计划于 8.0 移除） |
| `ccr list` | `ccr <claude\|codex\|grok> profile list` | 7.x 隐藏并弃用 |
| `ccr add` | `ccr <claude\|codex\|grok> profile create <name> ...` 或 `profile open` | 密钥请用 `--auth-token-stdin` / `--api-key-stdin` / `--env-key` |
| `ccr delete` / `enable` / `disable` | `ccr <claude\|codex\|grok> profile delete\|enable\|disable <name>` | 7.x 隐藏并弃用 |
| `ccr clear` / `ccr optimize` | `ccr <claude\|codex\|grok> profile off` / `profile open` | 退出 Profile 模式或在 `$EDITOR` 中编辑投影 |
| `ccr export` | `ccr <claude\|codex\|grok> profile export [--no-secrets] [-o file.toml]` | 导出 `ExchangeV1` TOML |
| `ccr import` | `ccr <claude\|codex\|grok> profile import <file.toml> [--dry-run] [--on-conflict skip\|overwrite\|rename] [--replace -y]` | 默认增量合并（旧 `ccr import` 默认整库替换） |

## Profile 存储与注册表迁移

### `profiles.toml` -> SQLite `profiles.db`

- Claude、Codex、Grok 在 7.x 中的权威活体配置库是 `~/.ccr/platforms/<platform>/profiles.db`（SQLite）。
- 首次访问旧版 `~/.ccr/platforms/<platform>/profiles.toml` 时，CCR 会自动将其迁移至 `profiles.db`，并将原 TOML 文件备份到 `backups/<platform>/`。
- TOML 格式现在仅用于 `ccr <claude|codex|grok> profile open` 的编辑器临时投影，以及 `profile export` / `profile import` 的 `ExchangeV1` 交换包；手工将备份 TOML 放回活路径不会被 7.x 二进制重新读取。

### 注册表字段兼容

- 旧版 `~/.ccr/config.toml` 可继续保留 `default_platform` / `current_platform` 字段，读取时保持兼容。
- 当前真实的路由指针是每个平台条目独立的 `current_profile` 字段。

## 用户心智迁移

旧心智：

- 先选一个“全局当前平台”
- 再用通用顶层子命令操作单个 `profiles.toml`

新心智：

- 用 `ccr current` 查看跨平台运行时状态总览
- 直接进入 `ccr <claude|codex|grok> profile ...` 或 `ccr <claude|codex|grok> auth ...`
- 官方登录账号（`auth`）与运行时配置库路由（`profile`）显式分离

## ccr-ui 使用统计迁移到 llmusage

ccr-ui 的 Usage Dashboard 已从 `ccr-db` 内置用量导入器迁移到外部 `llmusage` 运行时：导入/同步只调用已安装的 `llmusage` CLI，页面渲染只读 `llmusage` SQLite 数据库，不再链接上游 `llmusage` Rust crate。这个迁移只影响桌面端使用统计链路，不改变 Claude / Codex / Grok Profile、`SessionIndexer` 或预算/Stats 页面。旧的 `ccr-db` usage schema 会保留以兼容历史数据，但不再作为 Usage Dashboard 的新数据源。

### 数据位置

默认情况下，ccr-ui 使用与 llmusage CLI 对齐的 root 解析顺序：

1. `LLMUSAGE_HOME`
2. `~/.llmusage`

因此 ccr-ui 与 llmusage Web / CLI 默认读取同一个本地 SQLite root：

```text
~/.llmusage/llmusage.db
```

旧的 CCR 隔离目录（`~/.ccr/llmusage`）不会被自动读取、合并或迁移。Usage 诊断区会显示当前生效的 Archive Root，便于确认正在查询哪份数据。若需要刻意切换到其他数据目录，请在启动 ccr-ui 或 llmusage 前设置 `LLMUSAGE_HOME`：

```powershell
$env:LLMUSAGE_HOME = "D:\data\llmusage"
llmusage status
llmusage sync --rebuild --recent-days 30
```

### 首次启动与重新同步

- 打开 ccr-ui 的 Usage 页面后，导入按钮会执行 `llmusage --home <root> sync --json-events`，并把 CLI 事件桥接成既有 `usage:job-progress`、`usage:job-recent-ready`、`usage:job-finished`、`usage:job-failed` 事件。
- ccr-ui 保留 30 天 recent window 的默认导入行为；需要全量重建时，使用上面的 CLI 命令或后续专门的维护入口。
- `cache_savings`、双成本字段和日志 `recorded_at` 现在来自 ccr-ui 对 llmusage SQLite 表/视图的只读查询适配层，前端不再从 `total_cost` 或成本差值二次推导。

### 回滚与兼容边界

- 旧 `ccr-db` usage 表没有被删除，历史数据仍在原位置保留。
- 本次迁移不提供 `ccr-db -> llmusage` 历史数据搬迁器；如需补齐数据，请让 llmusage 重新解析本机 Claude / Codex / Gemini / OpenCode 原始日志。
- `ccr-store::CostTracker` 仍服务旧 Stats / 预算流程，不属于本次 Usage Dashboard 退役范围。
