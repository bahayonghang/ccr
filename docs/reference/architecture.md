# CCR 架构设计

> 面向当前 workspace 布局的权威说明。CCR 采用 Rust 2024 workspace，核心逻辑拆分到 `crates/`，UI、文档、脚本与示例保留在仓库根目录。

## 总览

- **核心 crate**：`crates/ccr`（可安装 CLI + 共享核心逻辑）
- **数据库层**：`crates/ccr-db`（SQLite、CheckIn、加密与相关服务）
- **共享类型**：`crates/ccr-types`（跨 crate/桌面端复用的数据结构）
- **UI 工程**：根 `ccr-ui/`（Vue 3 前端 + Tauri 桌面壳）
- **仓库根目录**：`docs/`、`scripts/`、`examples/` 保持根级；`outputs/` 用于汇总产物（如存在）

## 工作区结构

```text
ccr/
├── Cargo.toml                # workspace manifest + shared dependencies
├── crates/
│   ├── ccr/                  # installable CLI crate + shared runtime logic
│   │   ├── Cargo.toml
│   │   ├── src/              # cli / commands / services / managers / sync / web / tui
│   │   └── tests/            # integration tests for the CLI crate
│   ├── ccr-db/               # database-facing services and models
│   └── ccr-types/            # shared types reused across crates and desktop shell
├── ccr-ui/
│   ├── src/                  # Vue 3 application
│   ├── src-tauri/            # Tauri desktop shell
│   └── dist/                 # generated frontend assets after `ccr-ui` build
├── docs/                     # VitePress docs (zh/en)
├── scripts/                  # repository automation and maintenance helpers
├── examples/                 # sample configs and workflows
└── outputs/                  # collected/generated artifacts (optional)
```

## 分层设计

```text
CLI / Web API / TUI / Desktop shell
                ↓
         Commands / UI bridge
                ↓
          Services (编排)
                ↓
          Managers (持久化)
                ↓
         Core / Utils / Models
```

- **CLI 入口**：`crates/ccr/src/cli/` 与 `crates/ccr/src/main.rs`
- **命令实现**：`crates/ccr/src/commands/`
- **服务编排**：`crates/ccr/src/services/`
- **数据访问与持久化**：`crates/ccr/src/managers/`
- **平台实现**：`crates/ccr/src/platforms/`
- **基础设施**：`crates/ccr/src/core/`、`crates/ccr/src/utils/`
- **桌面壳集成**：`ccr-ui/src-tauri` 直接依赖 `crates/ccr`、`crates/ccr-db`、`crates/ccr-types`

## 依赖方向

- **严格单向**：接口层 → Commands → Services → Managers → Core/Utils
- **共享模块**：Models、Platforms、Utils 可被上层复用，但不反向依赖 UI
- **特性隔离**：`web` 与 `tui` 仍由 `crates/ccr` 的 feature flags 控制
- **桌面端复用**：Tauri 壳通过 crate 依赖复用核心逻辑，而不是依赖旧的 `ccr-ui/backend`

## 关键流程

### Profile 切换

1. `crates/ccr/src/cli/` 解析命令或快捷调用 `ccr <name>`
2. `ConfigService` 读取 `~/.ccr/config.toml` 与 `platforms/<name>/profiles.toml`
3. `SettingsService` 获取文件锁、执行备份并原子写入目标 `settings.json`
4. `HistoryService` 记录掩码后的差异
5. 需要时由 `TempOverrideManager` 注入临时 token/base_url/model

### WebDAV 同步

1. `sync config` 写入连接信息
2. `sync folder ...` 注册和启用同步目录
3. `SyncService` 递归处理 push/pull/all，过滤备份、历史、锁文件与 UI 缓存

### CCR UI 启动

1. `UiService` 优先探测本地 `./ccr-ui`
2. 若缺失则回退到用户目录 `~/.ccr/ccr-ui`
3. 仍不可用时，再提示从 GitHub 下载

## 可靠性与质量

- **并发安全**：文件锁 + 进程内互斥 + 原子写入
- **可恢复性**：切换、导入等破坏性操作前自动备份
- **日志**：`CCR_LOG_LEVEL` 控制，日志落在 `~/.ccr/logs/`
- **测试**：CLI 集成测试位于 `crates/ccr/tests/`
- **扩展性**：新增命令、平台与同步逻辑都以 `crates/ccr/src/` 为准扩展

## 扩展指南

### 新增命令

1. 在 `crates/ccr/src/commands/<domain>/` 创建模块
2. 在对应 `mod.rs` 导出命令函数
3. 在 `crates/ccr/src/cli/definitions.rs` 添加 CLI 定义
4. 在 `crates/ccr/src/cli/dispatch.rs` 接入路由

### 新增平台

1. 在 `crates/ccr/src/platforms/` 添加平台模块
2. 实现 `PlatformConfig` trait
3. 在 `crates/ccr/src/models/platform.rs` 中补充平台类型
4. 在 `crates/ccr/src/platforms/mod.rs` 注册工厂方法

## 参考文档

- [快速开始](/guide/quick-start)
- [命令参考](/reference/commands/)
- [迁移指南](/reference/migration)
