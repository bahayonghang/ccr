# Crate 地图

本页按 crate 和模块边界说明当前代码职责，作为 `architecture` 页的展开版。

## `crates/ccr`

### 入口与分发

- `main.rs`：程序入口、日志初始化、顶层错误处理
- `cli/definitions.rs`：CLI 结构与 `Commands` 枚举
- `cli/dispatch.rs`：命令分发、无子命令行为、`ccr ui` / `sync` / `codex` 等路由

### 用户可见命令层

- `commands/platform/`：平台注册表与平台切换
- `commands/profile/`：profile 生命周期
- `commands/lifecycle/`：init / clear / clean / optimize / validate
- `commands/data/`：history / export / import / stats / budget / pricing
- `commands/codex/`：Codex 专属命令，重点是 auth 与 env / quota
- `commands/sessions_cmd.rs`、`skills_cmd.rs`、`prompts_cmd.rs`：会话、skills、prompt 管理

### Shell / Compat 说明

- `services/ui_service.rs`：`ccr ui` 的探测、更新、下载与启动，属于 shell 能力
- `tui/`：终端 UI，属于 shell 能力
- `models/`、`managers/`、`services/` 中仍有部分旧域逻辑或 re-export facade
- 当 `ccr` 与 domain crate 同时提供同名能力时，以 domain crate 为事实源，`ccr` 视为兼容门面

## `crates/ccr-core`

- 共享基础设施：错误、锁、原子写入、日志、HTTP 等底层能力

## `crates/ccr-config`

- 平台类型、profile/settings 契约、平台注册表、配置 helper
- 作为配置与平台适配的主 owner

## `crates/ccr-store`

- history、本地 storage、session 索引与查询
- 作为 CLI/桌面端本地持久化与聚合查询的主 owner

## `crates/ccr-codex`

- Codex auth/runtime/quota/usage/session 专属逻辑

## `crates/ccr-sync`

- WebDAV 配置、目录注册、批量 push/pull 所需的 sync domain 能力

## `crates/ccr-skills`

- skills 安装/清单/cache/source 管理
- builtin prompts
- MCP preset 安装与跨平台同步

## `crates/ccr-db`

### 数据库入口

- `database/mod.rs`：数据库路径、全局连接池、迁移启动、事务包装
- `database/repositories/`：repository 层
- `database/schema.rs`、`migrations.rs`：schema 与数据迁移

### 业务域

- `models/checkin/`：签到域模型
- `managers/checkin/`：提供商、账号、记录、余额、导出、WAF cookie
- `services/checkin_service.rs`：签到执行与查询
- `services/usage_import_service.rs`：从 session 文件导入 usage
- `services/log_persistence.rs`：日志持久化

## `crates/ccr-types`

### 当前公开面

- `ClaudeSettings` 及其子结构：settings、hooks、MCP、slash commands、agents、plugins
- `LoginState`：Codex auth 状态表达
- `MonitoringEntry`、`FrontendLogInput`、`MonitoringFeedQuery`：监控输入输出

### 设计重点

- 所有结构都围绕 serde 兼容性设计
- 接受旧字段名或旧结构输入
- 用 `other` / `flatten` 保留未知字段

## 测试布局

- `crates/ccr/tests/commands.rs`
- `crates/ccr/tests/managers.rs`
- `crates/ccr/tests/platforms.rs`
- `crates/ccr/tests/workflows.rs`

这些入口再分发到对应子目录，用能力域而不是单文件堆叠方式组织集成测试。
