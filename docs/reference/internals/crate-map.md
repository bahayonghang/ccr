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

### 编排层

- `services/config_service.rs`：配置集合的增删改查、启停、导入导出
- `services/settings_service.rs`：settings 应用、备份、恢复、列举备份
- `services/codex_auth_service.rs`：Codex 多账号 auth、备份、切换、导入导出
- `services/sync_service.rs`：WebDAV 同步执行
- `services/ui_service.rs`：`ccr ui` 的探测、更新、下载与启动

### 持久化与配置层

- `managers/config/`、`platform_config.rs`：平台注册表与统一配置
- `managers/settings.rs`：settings 文件读写
- `managers/history.rs`：操作历史
- `managers/pricing_manager.rs`、`budget_manager.rs`、`cost_tracker.rs`：成本与预算
- `managers/skills_manager.rs`、`services/skills_service.rs`：skills 源、安装、清单、缓存
- `managers/mcp_preset_manager.rs`：MCP preset 安装与跨平台同步

### 会话与同步

- `sessions/`：session 文件解析、索引模型
- `storage/session_store.rs`：本地会话存储与查询
- `sync/`：WebDAV 配置、目录注册、批量 push/pull/status

### 共享基础设施

- `platforms/`：Claude / Codex / Gemini / Droid / Qwen 平台实现
- `core/`：错误、锁、原子写入、日志、HTTP 等基础设施
- `utils/`：mask、验证、格式转换等通用工具

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
- `LoginState`、`TokenFreshness`：Codex auth 状态表达
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
