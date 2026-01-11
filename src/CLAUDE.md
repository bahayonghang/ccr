# CCR Core CLI 模块指导文件

[根目录](../CLAUDE.md) > **src**

## Change Log
- **2026-01-11**: 补充 cli/, sessions/, storage/, sync/, platforms/, models/ 模块详细描述
- **2025-12-17**: 激进精简到 300 行以内，只保留核心架构和技术栈
- **2025-12-16**: 按标准模板重新组织文档结构
- **2025-10-22 00:04:36 CST**: 初始核心模块文档创建

---

## 项目架构

### 模块职责

`src/` 模块是 CCR 的核心 CLI 应用，提供完整的命令行界面、服务层、管理层和基础设施。

**核心功能**:
1. **CLI 接口** - 30+ 命令的完整命令行接口
2. **服务层** - 业务逻辑编排 (7 services)
3. **管理层** - 数据访问与持久化 (6+ managers)
4. **Web API** - 轻量级 Axum REST API (14 endpoints)
5. **TUI** - 交互式终端用户界面
6. **核心基础设施** - 错误处理、文件锁定、原子写入、日志
7. **Session 管理** - AI 会话解析、索引、搜索 (SQLite)
8. **平台抽象** - 6 平台支持 (Claude/Codex/Gemini/Qwen/IFlow/Droid)
9. **云端同步** - WebDAV 多文件夹同步

**设计特点**:
- 既是独立二进制 (`ccr`),也是可复用库
- 严格分层架构: CLI/Web → Services → Managers → Core/Utils
- 所有文件操作使用原子写入(临时文件 + 原子重命名)
- 文件锁定防止并发损坏
- 完整审计跟踪(UUID, 时间戳, 操作者)

### 架构层次

```
src/
├── CLI/Web Layer (命令层)
│   ├── main.rs              - CLI 入口 (Clap 解析)
│   ├── cli/                 - CLI 定义和分发
│   │   └── subcommands/     - 子命令模块 (check, codex, platform, sync, ui)
│   ├── commands/            - 30+ CLI 命令实现
│   ├── web/                 - Axum Web 服务器 (14 端点)
│   └── tui/                 - 终端 UI (Ratatui)
│
├── Service Layer (服务层)
│   ├── config_service.rs    - 配置操作编排
│   ├── settings_service.rs  - 设置管理
│   ├── history_service.rs   - 审计日志
│   ├── backup_service.rs    - 备份操作
│   ├── validate_service.rs  - 验证操作
│   ├── sync_service.rs      - WebDAV 同步
│   └── ui_service.rs        - UI 启动器
│
├── Manager Layer (管理层)
│   ├── config.rs            - 配置文件管理
│   ├── settings.rs          - Settings.json 管理
│   ├── history.rs           - 历史文件管理
│   ├── cost_tracker.rs      - 成本追踪管理
│   ├── budget.rs            - 预算控制管理
│   └── pricing.rs           - 价格表管理
│
├── Platform Layer (平台层)
│   ├── mod.rs               - 平台工厂和注册表
│   ├── base.rs              - 基础操作函数
│   ├── claude.rs            - Claude 平台实现
│   ├── codex.rs             - Codex 平台实现
│   ├── gemini.rs            - Gemini 平台实现
│   ├── qwen.rs              - Qwen 平台实现 (stub)
│   ├── iflow.rs             - IFlow 平台实现 (stub)
│   └── droid.rs             - Droid 平台实现
│
├── Session Layer (会话层)
│   ├── models.rs            - Session 数据模型
│   ├── parser.rs            - JSONL 解析器 (多平台)
│   └── indexer.rs           - 索引管理器 (SQLite)
│
├── Storage Layer (存储层)
│   ├── database.rs          - SQLite 数据库管理 (r2d2 连接池)
│   └── session_store.rs     - Session 存储层 (CRUD)
│
├── Sync Layer (同步层)
│   ├── config.rs            - 同步配置管理
│   ├── folder_manager.rs    - 多文件夹管理
│   ├── service.rs           - WebDAV 同步服务
│   ├── commands.rs          - 同步命令 (feature = "web")
│   └── content_selector.rs  - 内容选择器
│
├── Model Layer (模型层)
│   ├── platform.rs          - Platform 枚举和 PlatformConfig trait
│   ├── stats.rs             - 成本/Token 统计模型
│   ├── budget.rs            - 预算配置模型
│   └── pricing.rs           - 定价配置模型
│
└── Core/Utils Layer (核心层)
    ├── error.rs             - 自定义错误类型
    ├── lock.rs              - 文件锁定机制
    ├── atomic_writer.rs     - 原子文件写入
    ├── logging.rs           - 彩色输出
    ├── utils/validation.rs  - 验证辅助函数
    └── utils/mask.rs        - 敏感数据掩码
```

**关键原则**:
- **关注点分离**: 每层职责明确
- **原子操作**: 所有文件修改使用临时文件 + 原子重命名
- **并发安全**: 文件锁定防止多进程损坏
- **完整审计**: 每个操作都记录到历史文件
- **失败安全**: 破坏性操作前自动备份

---

## 项目技术栈

### 核心框架

| 技术 | 版本 | 用途 |
|------|------|------|
| **Rust** | Edition 2024 | 编程语言 (需要 1.85+) |
| **Clap** | 4.5+ | CLI 参数解析 (derive 宏) |
| **Tokio** | 1.48+ | 异步运行时 |
| **Axum** | 0.8+ | Web 框架 |

### 序列化与文件 I/O

| 技术 | 版本 | 用途 |
|------|------|------|
| **Serde** | 1.0+ | 序列化框架 |
| **serde_json** | 1.0+ | JSON 支持 |
| **toml** | 0.9+ | TOML 解析 |
| **indexmap** | 2.12+ | 有序 Map (保持配置顺序) |
| **dirs** | 6.0+ | 跨平台用户目录 |
| **fs4** | 0.13+ | 文件锁定 |
| **tempfile** | 3.23+ | 原子文件操作 |

### 错误处理与日志

| 技术 | 版本 | 用途 |
|------|------|------|
| **anyhow** | 1.0+ | 灵活错误处理 |
| **thiserror** | 2.0+ | 自定义错误宏 |
| **log** | 0.4+ | 日志 facade |
| **env_logger** | 0.11+ | 环境变量日志 |
| **colored** | 3.0+ | 彩色终端输出 |

### TUI 与 Web

| 技术 | 版本 | 用途 |
|------|------|------|
| **Ratatui** | 0.29+ | TUI 框架 |
| **Crossterm** | 0.29+ | 终端控制 |
| **comfy-table** | 7.2+ | 表格格式化 |
| **Tower-HTTP** | 0.6+ | CORS 中间件 |

### 工具库

| 技术 | 版本 | 用途 |
|------|------|------|
| **chrono** | 0.4+ | 日期时间 |
| **uuid** | 1.18+ | UUID 生成 |
| **whoami** | 2.0+ | 用户识别 |
| **sysinfo** | 0.37+ | 系统信息 |
| **reqwest_dav** | 0.2+ | WebDAV 客户端 |
| **blake3** | 1.8+ | 高性能哈希 (文件去重) |
| **rayon** | 1.11+ | 并行迭代器 |
| **rusqlite** | 0.38+ | SQLite 数据库 |
| **r2d2** | 0.8+ | 数据库连接池 |

---

## 项目模块划分

### 文件与文件夹布局

```
src/
├── main.rs                    # CLI 入口点 (Clap 解析 + 命令路由)
├── lib.rs                     # 库导出 (公开 API)
│
├── cli/                       # CLI 定义和分发
│   ├── mod.rs                 - CLI 主结构 (Clap derive)
│   └── subcommands/           - 子命令模块
│       ├── check.rs           - 检查命令
│       ├── codex.rs           - Codex 相关命令
│       ├── platform.rs        - 平台管理命令
│       ├── sync.rs            - 同步命令
│       └── ui.rs              - UI 命令
│
├── commands/                  # CLI 命令实现 (30+ 文件)
│   ├── mod.rs
│   ├── init.rs                - 初始化配置
│   ├── list.rs                - 列出配置
│   ├── current.rs             - 显示当前配置
│   ├── switch.rs              - 切换配置
│   ├── add.rs                 - 添加配置
│   ├── delete.rs              - 删除配置
│   ├── validate.rs            - 验证配置
│   ├── history_cmd.rs         - 历史记录
│   ├── export_import.rs       - 导入/导出
│   ├── clean.rs               - 清理备份
│   ├── sync_cmd.rs            - WebDAV 同步
│   ├── web_cmd.rs             - Web 服务器
│   ├── ui_cmd.rs              - UI 启动
│   ├── tui_cmd.rs             - TUI 启动
│   ├── stats_cmd.rs           - 成本统计
│   ├── budget_cmd.rs          - 预算管理
│   ├── pricing_cmd.rs         - 定价管理
│   ├── sessions_cmd.rs        - 会话管理
│   └── provider_cmd.rs        - Provider 健康检查
│
├── services/                  # 服务层 (7 文件)
│   ├── mod.rs
│   ├── config_service.rs      - 配置操作编排
│   ├── settings_service.rs    - 设置管理
│   ├── history_service.rs     - 审计日志
│   ├── backup_service.rs      - 备份操作
│   ├── validate_service.rs    - 验证操作
│   ├── sync_service.rs        - WebDAV 同步
│   └── ui_service.rs          - UI 启动器
│
├── managers/                  # 管理层 (6+ 文件)
│   ├── mod.rs
│   ├── config.rs              - 配置文件管理
│   ├── settings.rs            - Settings.json 管理
│   ├── history.rs             - 历史文件管理
│   ├── cost_tracker.rs        - 成本追踪管理
│   ├── budget.rs              - 预算控制管理
│   ├── pricing.rs             - 价格表管理
│   └── temp_override.rs       - 临时 Token 覆盖
│
├── platforms/                 # 平台实现 (6 平台)
│   ├── mod.rs                 - 平台工厂和注册表
│   ├── base.rs                - 基础操作函数
│   ├── claude.rs              - Claude 平台实现
│   ├── codex.rs               - Codex 平台实现
│   ├── gemini.rs              - Gemini 平台实现
│   ├── qwen.rs                - Qwen 平台实现 (stub)
│   ├── iflow.rs               - IFlow 平台实现 (stub)
│   └── droid.rs               - Droid 平台实现
│
├── sessions/                  # 会话管理
│   ├── mod.rs
│   ├── models.rs              - Session 数据模型
│   ├── parser.rs              - JSONL 解析器 (多平台)
│   └── indexer.rs             - 索引管理器 (SQLite)
│
├── storage/                   # 存储层
│   ├── mod.rs
│   ├── database.rs            - SQLite 数据库管理 (r2d2 连接池)
│   └── session_store.rs       - Session 存储层 (CRUD)
│
├── sync/                      # 同步模块
│   ├── mod.rs
│   ├── config.rs              - 同步配置管理
│   ├── folder_manager.rs      - 多文件夹管理
│   ├── service.rs             - WebDAV 同步服务
│   ├── commands.rs            - 同步命令 (feature = "web")
│   └── content_selector.rs    - 内容选择器
│
├── models/                    # 数据模型
│   ├── mod.rs
│   ├── platform.rs            - Platform 枚举和 PlatformConfig trait
│   ├── stats.rs               - 成本/Token 统计模型
│   ├── budget.rs              - 预算配置模型
│   └── pricing.rs             - 定价配置模型
│
├── core/                      # 核心基础设施
│   ├── mod.rs
│   ├── error.rs               - 自定义错误类型 (CcrError)
│   ├── lock.rs                - 文件锁定机制 (LockManager)
│   ├── atomic_writer.rs       - 原子文件写入
│   └── logging.rs             - 彩色输出 (ColorOutput)
│
├── web/                       # Web 服务器 (feature = "web")
│   ├── mod.rs
│   ├── server.rs              - Axum 服务器 (port 8080)
│   ├── routes.rs              - 路由定义 (14 端点)
│   └── handlers.rs            - API 处理器
│
├── tui/                       # 终端 UI (feature = "tui")
│   ├── mod.rs
│   ├── app.rs                 - TUI 应用状态
│   ├── ui.rs                  - UI 渲染
│   ├── event.rs               - 事件处理
│   └── tabs.rs                - 标签页 (Claude/Codex)
│
└── utils/                     # 工具函数
    ├── mod.rs
    ├── validation.rs          - 验证辅助函数 (Validatable trait)
    └── mask.rs                - 敏感数据掩码
```

### 核心入口点

| 入口文件 | 路径 | 职责 |
|----------|------|------|
| **CLI 入口** | `src/main.rs` | Clap 解析 + 命令路由 |
| **库入口** | `src/lib.rs` | 公开 API 导出 |
| **Web 入口** | `src/web/server.rs` | Axum 服务器 (port 8080) |
| **TUI 入口** | `src/tui/mod.rs` | 终端 UI 入口 |
| **Session 入口** | `src/sessions/mod.rs` | 会话解析和索引 |
| **Storage 入口** | `src/storage/mod.rs` | SQLite 数据库访问 |
| **Sync 入口** | `src/sync/mod.rs` | WebDAV 同步服务 |

---

## 项目代码风格与规范

### Rust 代码规范

#### 命名约定
- **模块名**: `snake_case` (如 `config_service`, `history_cmd`)
- **类型名**: `PascalCase` (如 `ConfigSection`, `CcrError`)
- **函数名**: `snake_case` (如 `switch_config`, `list_configs`)
- **常量**: `SCREAMING_SNAKE_CASE` (如 `DEFAULT_CONFIG`, `MAX_BACKUPS`)

#### 代码风格
- **Edition**: 2024 (需要 Rust 1.85+)
- **格式化**: 使用 `cargo fmt` (默认 rustfmt 设置)
- **检查**: 通过 `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used`
- **错误处理**: 使用 `CcrError` 类型,详细错误消息
- **文档**: 内部逻辑用中文注释,公开 API 用英文
- **测试代码规范**: 测试模块使用 `#[allow(clippy::unwrap_used)]` 属性允许 `unwrap()`（标准做法）

---

## 测试与质量

### 测试覆盖

- **目标**: 95%+ 整体覆盖率
- **单元测试**: 嵌入在模块中 (`#[cfg(test)]`)
- **集成测试**: `/tests/` 目录 (6 综合测试文件)
- **并发测试**: 多线程场景验证锁定

### 测试文件

位于 `/tests/`:
1. **integration_test.rs** - 核心集成测试
2. **manager_tests.rs** - 管理层测试
3. **service_workflow_tests.rs** - 服务层测试
4. **concurrent_tests.rs** - 并发与锁定测试
5. **end_to_end_tests.rs** - 完整工作流测试
6. **add_delete_test.rs** - 配置 CRUD 操作

### 运行测试

```bash
# 所有测试
cargo test

# 特定测试文件
cargo test --test concurrent_tests

# 带输出
cargo test -- --nocapture

# 单个测试
cargo test test_switch_config
```

### 质量检查

```bash
# 代码检查 (标准)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 代码检查 (严格，对齐 CI)
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used

# 格式化检查
cargo fmt --check

# 安全审计
cargo audit
```

---

## 项目构建、测试与运行

### 环境要求

- **Rust**: 1.85+ (Edition 2024)
- **Cargo**: 最新稳定版

### 开发命令

```bash
# 构建
cargo build                    # Debug 构建
cargo build --release          # Release 构建

# 运行
cargo run                      # 运行 Debug 版本
cargo run --release            # 运行 Release 版本

# 测试
cargo test                     # 运行所有测试
cargo clippy                   # Lint
cargo fmt                      # 格式化

# 环境变量调试
export CCR_LOG_LEVEL=debug     # 设置日志级别
                               # (trace|debug|info|warn|error)
```

### 使用 Justfile

项目根目录的 `justfile` 提供快捷命令:

```bash
just build                     # Debug 构建
just release                   # Release 构建
just test                      # 运行测试
just lint                      # Format + Clippy (标准)
just lint-strict               # Format + Clippy (严格：禁止 unwrap)
just ci                        # 完整 CI 流程 (使用严格 Clippy)
```

### 安装

```bash
# 从 GitHub 安装
cargo install --git https://github.com/bahayonghang/ccr ccr

# 从源码构建
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .

# 初始化配置
ccr init
```

---

## Git 工作流程

### 分支策略

- **main**: 主分支,生产环境代码
- **dev**: 开发分支,测试环境代码
- **feature/***: 功能分支
- **bugfix/***: Bug 修复分支

### 提交规范

遵循 Conventional Commits:

```bash
# 功能开发
git commit -m "feat(CLI): 添加 platform 命令"
git commit -m "feat(服务): 实现 WebDAV 多文件夹同步"

# Bug 修复
git commit -m "fix(CLI): 修复配置切换时的锁定问题"
git commit -m "fix(管理器): 修复 TOML 解析错误"

# 重构
git commit -m "refactor(核心): 重构错误处理使用 thiserror"

# 性能优化
git commit -m "perf(文件): 优化文件读取性能"

# 文档
git commit -m "docs(README): 更新安装说明"

# 测试
git commit -m "test(集成): 添加并发测试"
```

---

## 文档目录

### 文档存储规范

- **模块文档**: `/src/CLAUDE.md` (本文件)
- **根文档**: `/CLAUDE.md` (项目总览)
- **UI 文档**: `/ccr-ui/CLAUDE.md` (CCR UI 总览)

---

