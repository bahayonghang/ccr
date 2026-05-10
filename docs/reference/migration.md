# 迁移指南

本页说明从旧的全局平台 / profile 路由模型迁移到显式 Claude Runtime / Codex Runtime 模型。

## 命令迁移速查表

| 旧命令 | 当前做法 | 说明 |
|---|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` / `ccr codex profile switch <name>` | 不再隐式推断平台 |
| `ccr <name>` | 同上 | 快捷入口已退休 |
| `ccr platform switch <platform>` | 不再作为 auth/profile 主路径 | 改用显式 profile/auth 命令 |
| `ccr platform current` | `ccr current` | 查看双 runtime |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` | 平台级 profile 命令已显式分离 |

## registry 迁移

- 旧文件可继续包含 `default_platform` / `current_platform`
- 读取时仍兼容这些字段
- 但当前路由真相已经变为各平台条目的 `current_profile`

## 用户心智迁移

旧心智：

- 先选一个“当前平台”
- 再用通用 `switch` 切 profile

新心智：

- `ccr current` 先看 Claude / Codex 双 runtime 状态
- 直接进入 `ccr claude profile ...` 或 `ccr codex profile ...`
- official auth 与 profile routing 分开表达

## ccr-ui 使用统计迁移到 llmusage

ccr-ui 的 Usage Dashboard 已从 `ccr-db` 内置用量导入器迁移到 `llmusage` 0.5.1 运行时。这个迁移只影响桌面端使用统计链路，不改变 Claude / Codex profile、SessionIndexer 或预算/Stats 页面。旧的 `ccr-db` usage schema 会保留以兼容历史数据，但不再作为 Usage Dashboard 的新数据源。

### 数据位置

默认情况下，ccr-ui 遵循 llmusage 标准的 `AppPaths::discover()` 解析顺序：

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

- 打开 ccr-ui 的 Usage 页面后，导入按钮会触发 llmusage `JobRegistry`，并继续发出既有 `usage:job-progress`、`usage:job-recent-ready`、`usage:job-finished`、`usage:job-failed` 事件。
- ccr-ui 保留 30 天 recent window 的默认导入行为；需要全量重建时，使用上面的 CLI 命令或后续专门的维护入口。
- `cache_savings`、双成本字段和日志 `recorded_at` 现在由 llmusage 后端适配层直接提供，前端不再从 `total_cost` 或成本差值二次推导。

### 回滚与兼容边界

- 旧 `ccr-db` usage 表没有被删除，历史数据仍在原位置保留。
- 本次迁移不提供 `ccr-db -> llmusage` 历史数据搬迁器；如需补齐数据，请让 llmusage 重新解析本机 Claude / Codex / Gemini / OpenCode 原始日志。
- `ccr-store::CostTracker` 仍服务旧 Stats / 预算流程，不属于本次 Usage Dashboard 退役范围。
