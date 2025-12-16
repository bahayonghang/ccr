# CCR Core CLI 模块指导文件

[根目录](../CLAUDE.md) > **src**

## Change Log
- **2025-12-16**: 按标准模板重新组织文档结构
- **2025-10-22 00:04:36 CST**: 初始核心模块文档创建

---

## 项目架构

### 模块职责

`src/` 模块是 CCR 的核心 CLI 应用,实现主要的配置管理逻辑。提供完整的命令行界面、服务层、管理层和基础设施。

**核心功能**:
1. **CLI 接口** - 13+ 命令的完整命令行接口
2. **服务层** - 业务逻辑编排 (6 services)
3. **管理层** - 数据访问与持久化 (3 managers)
4. **Web API** - 轻量级 Axum REST API (14 endpoints)
5. **TUI** - 交互式终端用户界面
6. **核心基础设施** - 错误处理、文件锁定、原子写入、日志

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
│   ├── commands/            - 13+ CLI 命令实现
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
│   └── history.rs           - 历史文件管理
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
| **whoami** | 1.6+ | 用户识别 |
| **sysinfo** | 0.37+ | 系统信息 |
| **reqwest_dav** | 0.2+ | WebDAV 客户端 |

---

## 项目模块划分

### 文件与文件夹布局

```
src/
├── main.rs                    # CLI 入口点
├── lib.rs                     # 库导出
│
├── commands/                  # CLI 命令 (13+ 文件)
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
│   └── tui_cmd.rs             - TUI 启动
│
├── services/                  # 服务层 (6 文件)
│   ├── mod.rs
│   ├── config_service.rs      - 配置操作
│   ├── settings_service.rs    - 设置管理
│   ├── history_service.rs     - 审计日志
│   ├── backup_service.rs      - 备份操作
│   ├── validate_service.rs    - 验证操作
│   ├── sync_service.rs        - WebDAV 同步
│   └── ui_service.rs          - UI 启动器
│
├── managers/                  # 管理层 (3 文件)
│   ├── mod.rs
│   ├── config.rs              - 配置文件管理
│   ├── settings.rs            - Settings.json 管理
│   └── history.rs             - 历史文件管理
│
├── core/                      # 核心基础设施 (5 文件)
│   ├── mod.rs
│   ├── error.rs               - 自定义错误类型
│   ├── lock.rs                - 文件锁定机制
│   ├── atomic_writer.rs       - 原子文件写入
│   └── logging.rs             - 彩色输出
│
├── web/                       # Web 服务器 (4 文件)
│   ├── mod.rs
│   ├── server.rs              - Axum 服务器 (port 8080)
│   ├── routes.rs              - 路由定义
│   └── handlers.rs            - API 处理器
│
├── tui/                       # 终端 UI (5 文件)
│   ├── mod.rs
│   ├── app.rs                 - TUI 应用状态
│   ├── ui.rs                  - UI 渲染
│   ├── event.rs               - 事件处理
│   └── tabs.rs                - 标签页
│
├── utils/                     # 工具函数 (3 文件)
│   ├── mod.rs
│   ├── validation.rs          - 验证辅助函数
│   └── mask.rs                - 敏感数据掩码
│
└── models/                    # 数据模型
    └── mod.rs
```

### 核心入口点

| 入口文件 | 路径 | 职责 |
|----------|------|------|
| **CLI 入口** | `src/main.rs` | Clap 解析 + 命令路由 |
| **库入口** | `src/lib.rs` | 公开 API 导出 |
| **Web 入口** | `src/web/server.rs` | Axum 服务器 (port 8080) |
| **TUI 入口** | `src/tui/mod.rs` | 终端 UI 入口 |

---

## 项目业务模块

### 1. 配置管理命令

**命令**: `ccr init`, `list`, `switch`, `add`, `delete`

**功能**:
- 初始化配置文件
- 列出所有配置(表格格式)
- 显示当前配置与环境
- 切换到指定配置
- 交互式添加新配置
- 删除配置(带确认或强制)

### 2. 操作与历史

**命令**: `ccr validate`, `history`, `export`, `import`, `clean`

**功能**:
- 验证所有配置和设置
- 查看操作历史(可过滤)
- 导出配置(可隐藏敏感信息)
- 导入配置(合并或强制覆盖)
- 清理旧备份(按天数或 dry-run)

### 3. 云端同步

**命令**: `ccr sync config`, `sync status`, `sync push`, `sync pull`

**功能**:
- 配置 WebDAV 连接
- 检查同步状态
- 推送配置到云端
- 从云端拉取配置

### 4. 用户界面

**命令**: `ccr tui`, `web`, `ui`

**功能**:
- **TUI**: 终端 UI (Ratatui + Crossterm)
- **Web**: 轻量级 API 服务器 (port 8080)
- **UI**: 完整 Web 应用 (port 3000 + 8081)

### 5. 系统工具

**命令**: `ccr update`, `version`

**功能**:
- 从 GitHub 更新 CCR
- 显示版本和功能信息

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
- **检查**: 通过 `cargo clippy` 无警告
- **错误处理**: 使用 `CcrError` 类型,详细错误消息
- **文档**: 内部逻辑用中文注释,公开 API 用英文

#### 错误处理示例

```rust
use crate::core::error::CcrError;

pub fn read_config() -> Result<Config, CcrError> {
    let content = std::fs::read_to_string("config.toml")
        .map_err(|e| CcrError::FileReadError(e.to_string()))?;

    let config: Config = toml::from_str(&content)
        .map_err(|e| CcrError::ParseError(e.to_string()))?;

    Ok(config)
}
```

每个错误映射到特定退出码:
- `ConfigNotFound` → 退出码 2
- `ValidationError` → 退出码 3
- `LockError` → 退出码 4
- Fatal errors → 退出码 1

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

# 覆盖率报告 (需要 tarpaulin)
cargo tarpaulin --out Html
```

### 质量检查

```bash
# 代码检查
cargo clippy --all-targets --all-features

# 格式化检查
cargo fmt --check

# 安全审计
cargo audit

# 依赖树
cargo tree
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
just lint                      # Format + Clippy
just ci                        # 完整 CI 流程
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

## 文档目录(重要)

### 文档存储规范

- **模块文档**: `/src/CLAUDE.md` (本文件)
- **根文档**: `/CLAUDE.md` (项目总览)
- **UI 文档**: `/ccr-ui/CLAUDE.md` (CCR UI 总览)

### 相关文件列表

#### 源代码
- `/src/main.rs` - CLI 入口
- `/src/lib.rs` - 库导出
- `/src/commands/*.rs` - CLI 命令 (13 文件)
- `/src/services/*.rs` - 服务层 (6 文件)
- `/src/managers/*.rs` - 管理层 (3 文件)
- `/src/core/*.rs` - 核心层 (5 文件)
- `/src/web/*.rs` - Web 服务器 (4 文件)
- `/src/tui/*.rs` - Terminal UI (5 文件)
- `/src/utils/*.rs` - 工具函数 (3 文件)

#### 配置文件
- `/Cargo.toml` - Rust 依赖
- `/.gitignore` - Git 忽略规则

#### 测试
- `/tests/*.rs` - 集成测试 (6 文件)

#### 文档
- `/README.md` - 项目 README
- `/README_CN.md` - 中文 README

### 外部链接

- **Rust Book**: https://doc.rust-lang.org/book/
- **Clap 文档**: https://docs.rs/clap/
- **Tokio 文档**: https://docs.rs/tokio/
- **Axum 文档**: https://docs.rs/axum/
- **Ratatui 文档**: https://docs.rs/ratatui/

---

## 常见问题(FAQ)

### Q: CCR 如何确保并发安全?

A: CCR 使用基于文件的锁定 (`fs4` crate),自动获取和释放锁。每个操作在修改文件前获取锁,防止多进程同时访问导致损坏。

### Q: 如果 CCR 在配置切换时崩溃会怎样?

A: 所有文件写入使用原子操作(写入临时文件 → 重命名)。如果进程崩溃,原始文件保持不变。另外,破坏性操作前自动创建备份。

### Q: API 密钥如何保护?

A: API 密钥在所有输出(日志、历史、显示)中使用模式匹配掩码。掩码逻辑在 `src/utils/mask.rs`。

### Q: CCR 和 CCS 可以一起使用吗?

A: 可以!两个工具共享同一个 `~/.ccs_config.toml` 文件,可以共存。它们使用不同的锁定机制,不会互相干扰。

### Q: 备份存储在哪里?

A: 自动备份在 `~/.claude/backups/`,带时间戳命名如 `settings_20250101_120000.json.bak`。

### Q: 如何启用调试日志?

A: 设置环境变量: `export CCR_LOG_LEVEL=debug` (或运行前 `CCR_LOG_LEVEL=debug ccr <command>`)。

### Q: `ccr web` 和 `ccr ui` 的区别?

A:
- `ccr web` - 轻量级 API 服务器 (14 端点, port 8080) 用于编程访问
- `ccr ui` - 完整 Web 应用 (129 端点后端 + Vue 前端, ports 8081/3000) 用于可视化管理

---

**本小姐精心整理的核心 CLI 模块文档完成！分层架构清晰,这才是 CCR 的灵魂所在呢～(￣▽￣)／**
