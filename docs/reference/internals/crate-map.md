# Crate 地图

本页列出根 `Cargo.toml` 的全部 workspace member。crate 清单由文档审计与 workspace 自动对照。

## 入口与交互

### `crates/ccr`

- 生成可安装的 `ccr` 二进制。
- 组装 `ccr-cli` dispatcher 与可选 `ccr-tui` launcher。
- 保留少量兼容 re-export；不再是所有领域逻辑的所有者。

### `crates/ccr-cli`

- `src/cli/definitions.rs`：顶层 Clap `Commands`。
- `src/cli/subcommands/`：Claude、Codex、OpenCode、platform、sync 等嵌套命令。
- `src/cli/dispatch.rs`：命令路由、TUI launcher 与 legacy 路径处理。
- `src/commands/`：用户可见处理器；`services/`、`managers/`、`platforms/` 负责 CLI 专属编排。

### `crates/ccr-tui`

- Ratatui 渲染、交互状态和平台标签页。
- 复用 `ccr-cli`、`ccr-codex` 和 `ccr-usage`，不复制命令或 usage 领域。

## 共享基础与契约

### `crates/ccr-core`

- 共享错误、路径、锁、原子写、日志、HTTP 和基础 domain model。
- 为上层 crate 提供安全写入与通用基础设施。

### `crates/ccr-types`

- Claude settings、Codex login state、监控/日志 payload 等共享 serde 类型。
- 通过 alias、flatten 和 unknown-field 保留维持旧数据兼容。

## 配置与持久化

### `crates/ccr-config`

- 平台枚举、profile/settings 契约、平台注册表和格式转换。
- Claude、Codex、Antigravity、Droid 与 Qwen stub 的统一配置边界。

### `crates/ccr-store`

- CLI history 和 session SQLite 存储。
- session 索引、搜索、统计与清理查询。

### `crates/ccr-db`

- 桌面 SQLite 连接、migration、repository 与事务。
- Check-in 数据、日志和其他桌面服务的数据层。

## 领域 crate

### `crates/ccr-codex`

- Codex auth snapshot、profile/runtime、quota、usage 与 session 领域。

### `crates/ccr-sync`

- WebDAV 配置、folder registry、push/pull/status 与批量同步基础。

### `crates/ccr-skills`

- skills source/inventory/install/cache、builtin prompts 与 MCP presets。

### `crates/ccr-checkin`

- Check-in 业务门面，组合 provider、account、balance、记录与执行服务。
- 数据持久化复用 `ccr-db`，共享契约复用 `ccr-types`。

### `crates/ccr-usage`

- 读取 llmusage SQLite/CLI 能力并输出稳定只读投影。
- `ts` feature 为 Tauri/Vue 边界导出 TypeScript 类型。
- 不直接解析原始 transcript，也不拥有上游 schema migration。

## 前端消费者

`ccr-ui/src-tauri` 直接依赖除 `ccr-tui` 外的 workspace domain crates，并通过 `commands/handler_registry.rs` 暴露 invoke handlers。Vue 端通过 `src/api/domains/*` 消费这些命令。

## 测试入口

- crate 单元测试位于各自 `src/` 模块或 `tests/`。
- CLI 集成测试位于 `crates/ccr/tests/commands/` 等能力目录。
- TUI 测试属于 `ccr-tui`。
- usage TypeScript binding 和 projection 测试属于 `ccr-usage`。
- Tauri 集成测试位于 `ccr-ui/src-tauri/tests/`。

## 相关页面

- [架构](../architecture)
- [运行时流程](./runtime-flows)
