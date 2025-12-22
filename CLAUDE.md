<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# CCR 项目开发指南

> CCR (Claude Code Configuration Switcher) - 多平台 AI CLI 配置管理工具
>
> 最后更新：2025-12-17

---

## 项目架构

### 核心架构

CCR 采用严格的分层架构：

```
CLI/Web Layer → Services Layer → Managers Layer → Core/Utils Layer
```

**核心原则**：
- 关注点分离：每层职责明确，依赖单向流动
- 原子操作：所有文件修改使用临时文件 + 原子重命名
- 并发安全：文件锁定机制防止多进程冲突
- 完整审计：所有操作记录到历史文件
- 失败安全：破坏性操作前自动备份

### 模块组织

```
ccr/
├── src/                # 核心 CLI 模块 (详见 src/CLAUDE.md)
│   ├── commands/       # 13+ CLI 命令
│   ├── services/       # 6 业务服务
│   ├── managers/       # 3 数据管理器
│   ├── core/           # 错误、锁定、日志
│   ├── web/            # Axum Web 服务器 (14 端点)
│   └── tui/            # 终端 UI (Ratatui)
│
├── ccr-ui/             # 全栈 Web 应用 (详见 ccr-ui/CLAUDE.md)
│   ├── backend/        # Axum 后端 (129 端点)
│   └── frontend/       # Vue.js 3 SPA
│
└── tests/              # 集成测试 (95%+ 覆盖率)
```

---

## 技术栈

### 后端 (Rust)

| 类别 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 语言 | Rust | 2024 Edition | 核心语言 (需要 1.85+) |
| CLI | Clap | 4.5+ | 参数解析 |
| 异步 | Tokio | 1.48+ | 异步运行时 |
| Web | Axum | 0.8+ | Web 框架 |
| TUI | Ratatui | 0.29+ | 终端 UI |
| 序列化 | Serde | 1.0+ | JSON/TOML/YAML |
| 错误 | Anyhow/Thiserror | 1.0+/2.0+ | 错误管理 |

### 前端 (TypeScript/Vue)

| 类别 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 框架 | Vue.js | 3.5.22 | UI 框架 |
| 构建 | Vite | 7.1.11 | 构建工具 |
| 路由 | Vue Router | 4.4.5 | 路由管理 |
| 状态 | Pinia | 2.2.6 | 状态管理 |
| 样式 | Tailwind CSS | 3.4.17 | CSS 框架 |
| HTTP | Axios | 1.7.9 | API 客户端 |
| 类型 | TypeScript | 5.7.3 | 类型安全 |

---

## 核心功能

### 1. 配置管理

**命令**：`ccr init/list/switch/add/delete/validate/history`

- 管理 CCR 配置段 (列出/切换/添加/编辑/删除)
- 验证、导入/导出配置
- 完整操作历史记录

### 2. 多平台支持

**命令**：`ccr platform init/list/switch/current/info`

- ✅ **Claude Code**：MCP、Agents、斜杠命令、插件
- ✅ **Codex**：配置文件、MCP、Agents、斜杠命令、插件
- ✅ **Gemini CLI**：同上
- 🚧 **Qwen/iFlow**：计划中

### 3. WebDAV 云端同步

**命令**：`ccr sync config/folder/push/pull/status`

- 多文件夹独立同步 (v2.5+)
- 单文件夹/批量操作
- 自动迁移 v2.4 → v2.5

### 4. 用户界面

- **CLI**：命令行工具 (13+ 命令)
- **TUI**：终端 UI (`ccr tui`)
- **Web API**：轻量级 API (14 端点, port 8080)
- **Web UI**：完整应用 (129 端点后端 + Vue 前端, `ccr ui`)

---

## 代码规范

### 命名约定

**Rust**：
- 模块名：`snake_case`
- 类型名：`PascalCase`
- 函数名：`snake_case`
- 常量：`SCREAMING_SNAKE_CASE`

**TypeScript/Vue**：
- 组件：`PascalCase`
- 变量/函数：`camelCase`
- 常量：`SCREAMING_SNAKE_CASE`
- 类型：`PascalCase`

### 代码风格

**Rust**：
- 使用 `Result` 类型与 `?` 操作符
- 自定义错误类型 `CcrError`/`AppError`
- 内部注释用中文，公开 API 用英文
- 格式化：`cargo fmt`，检查：`cargo clippy`

**TypeScript/Vue**：
- 使用 `<script setup>` Composition API
- TypeScript 严格模式
- Try-catch 异常处理
- Tailwind CSS 优先
- 格式化：`npm run format`，检查：`npm run lint`

### Import 顺序

**Rust**：标准库 → 外部 crate → 内部模块

**TypeScript**：Vue 核心 → 第三方库 → 类型定义 → API 客户端 → 组件

---

## 构建与运行

### 环境要求

- **Rust**: 1.85+ (Edition 2024)
- **Node.js**: 18.x+
- **Cargo**: 最新稳定版
- **npm/yarn/pnpm**: 9.x+

### 快速启动

#### 使用 Just (推荐)

```bash
# 项目根目录
just build          # 构建 Debug
just release        # 构建 Release
just test           # 运行测试
just lint           # Format + Clippy
just ci             # 完整 CI 流程

# ccr-ui/ 目录
cd ccr-ui
just i              # 安装依赖
just s              # 启动开发环境
just b              # 构建生产版本
just c              # 代码检查
```

#### 手动命令

**核心 CLI**：
```bash
cargo build                    # Debug 构建
cargo run                      # 运行 CLI
cargo build --release          # Release 构建
cargo test                     # 运行测试
```

**UI 后端**：
```bash
cd ccr-ui/backend
cargo run                      # 启动后端 (port 8081)
RUST_LOG=debug cargo run       # 启用调试日志
```

**UI 前端**：
```bash
cd ccr-ui/frontend
npm install                    # 安装依赖
npm run dev                    # 启动开发服务器 (port 3000)
npm run build                  # 生产构建
```

### 环境变量

**Rust (CLI/后端)**：
```bash
CCR_LOG_LEVEL=debug            # trace|debug|info|warn|error
RUST_LOG=debug                 # 后端日志
HOST=127.0.0.1                 # 绑定地址
PORT=8081                      # 监听端口
```

**前端 (.env)**：
```bash
VITE_API_BASE_URL=http://localhost:8081  # 开发环境
VITE_API_BASE_URL=/api                   # 生产环境
```

---

## 测试

### 运行测试

```bash
# Rust
cargo test                     # 所有测试
cargo test --test integration_test
cargo test -- --nocapture      # 带输出

# TypeScript
npm run type-check             # 类型检查
npm run lint                   # 代码检查
```

### 测试文件

**位置**：`/tests/`
- `integration_test.rs` - 核心集成
- `manager_tests.rs` - 管理层测试
- `concurrent_tests.rs` - 并发测试
- `service_workflow_tests.rs` - 服务层测试
- `end_to_end_tests.rs` - 端到端测试
- `add_delete_test.rs` - CRUD 操作

### 质量目标

- ✅ 零编译/类型错误
- ✅ 零 Clippy/ESLint 警告
- ✅ 代码格式化
- 🎯 测试覆盖率 95%+ (核心 CLI 已达成)

---

## Git 工作流程

### 分支策略

- **main**: 生产环境
- **dev**: 开发环境
- **feature/***: 功能分支
- **bugfix/***: Bug 修复

### 提交规范 (Conventional Commits)

```bash
feat(CLI): 添加 platform 命令
fix(UI): 修复暗黑模式样式
docs(CLAUDE): 精简项目指导文档
refactor(后端): 重构为分层架构
perf(核心): 优化文件读取性能
test(集成): 添加并发测试
```

**格式**：`<type>(<scope>): <subject>`

**类型**：`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

---

## 文档结构

```
ccr/
├── CLAUDE.md                  # 本文件 - 项目总览
├── src/CLAUDE.md              # 核心 CLI 模块文档
├── ccr-ui/CLAUDE.md           # UI 总览文档
├── ccr-ui/backend/CLAUDE.md   # 后端模块文档
└── ccr-ui/frontend/CLAUDE.md  # 前端模块文档
```

---
