# CCR UI Backend 模块指导文件

[根目录](../../CLAUDE.md) > [ccr-ui](../CLAUDE.md) > **backend**

## Change Log
- **2025-12-16**: 按标准模板重新组织文档结构
- **2025-11-14**: 重构为分层架构 (API → Services → Managers → Models → Core → Utils)
- **2025-10-22 10:39:28 CST**: 初始后端模块文档创建

---

## 项目架构

### 模块职责

CCR UI Backend 是基于 Axum 构建的 REST API 服务器,为多个 AI CLI 工具提供全面的管理接口。

**核心职责**:
1. **多平台配置管理** - Claude Code, Codex, Gemini CLI, Qwen, iFlow
2. **MCP 服务器管理** - 列表、添加、更新、删除、启用/禁用 MCP 服务器
3. **Agent 管理** - 管理各平台的 AI Agents
4. **斜杠命令管理** - 自定义斜杠命令配置
5. **插件管理** - 插件安装与配置
6. **多文件夹 WebDAV 同步** - 独立文件夹管理与批量操作
7. **配置转换** - 平台间配置格式转换
8. **命令执行** - 通过 API 执行 CCR CLI 命令
9. **系统信息** - 提供系统指标与状态

**运行环境**:
- 独立 Axum 服务器,默认端口 **8081**
- 通过 RESTful JSON APIs 与前端通信
- 通过子进程执行 CCR CLI 命令(不重复实现逻辑)

### 架构层次

**新分层架构** (2025-11-14 重构):

```
backend/
├── API Layer (API 层)           - HTTP 请求处理、路由定义、JSON 解析
│   └── handlers/                - 16+ 处理器文件
│
├── Services Layer (服务层)      - 业务逻辑、编排、事务管理
│   ├── commands.rs              - 命令服务
│   └── converter_service.rs     - 转换服务
│
├── Managers Layer (管理层)      - 数据访问、文件 I/O、持久化操作
│   ├── settings_manager.rs      - 设置持久化
│   ├── markdown_manager.rs      - Markdown 文件处理
│   ├── plugins_manager.rs       - 插件管理
│   └── config/                  - 配置文件管理器
│       ├── claude_manager.rs
│       ├── codex_manager.rs
│       ├── gemini_manager.rs
│       ├── qwen_manager.rs
│       └── platform_manager.rs
│
├── Models Layer (模型层)        - 数据结构、序列化、验证
│   ├── api.rs                   - API 模型 (MCP, Agent, etc.)
│   ├── converter.rs             - 转换模型
│   └── platforms/               - 平台特定模型
│
├── Core Layer (核心层)          - 基础设施 (错误、命令执行)
│   ├── error.rs                 - 错误类型与处理
│   └── executor.rs              - CCR 命令执行器
│
└── Utils Layer (工具层)         - 通用工具与辅助函数
    └── config_reader.rs         - 通用配置工具
```

**关键原则**:
- **严格单向依赖**: API → Services → Managers → Models/Core/Utils
- **无循环依赖**: 依赖只能向下流动
- **关注点分离**: 每层职责明确
- **原子文件操作**: Managers 层使用临时文件 + 原子重命名
- **平台隔离**: 平台特定代码在子模块中隔离

---

## 项目技术栈

### 核心框架

| 技术 | 版本 | 用途 |
|------|------|------|
| **Rust** | 2024 Edition | 编程语言 |
| **Axum** | 0.7+ | Web 框架 |
| **Tokio** | 1.42+ | 异步运行时 |
| **Tower** | 0.5+ | 服务抽象层 |
| **Tower-HTTP** | 0.6+ | HTTP 中间件 (CORS, 压缩, 追踪) |

### 序列化与解析

| 技术 | 版本 | 用途 |
|------|------|------|
| **Serde** | 1.0+ | 序列化框架 |
| **serde_json** | 1.0+ | JSON 支持 |
| **toml** | 0.9+ | TOML 解析 |
| **serde_yaml** | 0.9+ | YAML 解析 |
| **chrono** | 0.4+ | 日期时间处理 |

### 错误处理与日志

| 技术 | 版本 | 用途 |
|------|------|------|
| **anyhow** | 1.0+ | 灵活错误处理 |
| **thiserror** | 2.0+ | 自定义错误宏 |
| **tracing** | 0.1+ | 结构化日志 |
| **tracing-subscriber** | 0.3+ | 日志订阅器 |
| **tracing-appender** | 0.2+ | 日志文件轮换 |

### CLI 与系统

| 技术 | 版本 | 用途 |
|------|------|------|
| **clap** | 4.5+ | 命令行参数解析 |
| **whoami** | 1.5+ | 用户识别 |
| **num_cpus** | 1.16+ | CPU 核心数 |
| **sysinfo** | 0.32+ | 系统信息 |

### HTTP 客户端

| 技术 | 版本 | 用途 |
|------|------|------|
| **reqwest** | 0.12+ | HTTP 客户端 |

---

## 项目模块划分

### 文件与文件夹布局

```
ccr-ui/backend/
├── src/
│   ├── main.rs                              # 入口点与路由配置
│   │
│   ├── api/                                 # API 层
│   │   ├── mod.rs
│   │   └── handlers/
│   │       ├── mod.rs                       # Handler 导出
│   │       ├── config.rs                    # CCR 配置端点
│   │       ├── command.rs                   # 命令执行端点
│   │       ├── system.rs                    # 系统信息端点
│   │       ├── version.rs                   # 版本端点
│   │       ├── sync.rs                      # WebDAV 同步端点
│   │       ├── mcp.rs                       # Claude MCP 端点
│   │       ├── agents.rs                    # Claude Agents 端点
│   │       ├── slash_commands.rs            # Claude 斜杠命令
│   │       ├── plugins.rs                   # Claude 插件端点
│   │       ├── converter.rs                 # 配置转换端点
│   │       ├── platform.rs                  # 平台管理端点
│   │       ├── stats.rs                     # 统计端点
│   │       ├── budget.rs                    # 预算管理端点
│   │       ├── pricing.rs                   # 定价管理端点
│   │       └── platforms/                   # 平台特定处理器
│   │           ├── mod.rs
│   │           ├── codex.rs                 # Codex 端点
│   │           ├── gemini.rs                # Gemini CLI 端点
│   │           ├── qwen.rs                  # Qwen 端点
│   │           └── iflow.rs                 # iFlow 端点 (stub)
│   │
│   ├── services/                            # 服务层
│   │   ├── mod.rs
│   │   ├── commands.rs                      # 命令服务
│   │   └── converter_service.rs             # 转换服务
│   │
│   ├── managers/                            # 管理层
│   │   ├── mod.rs
│   │   ├── settings_manager.rs              # 设置持久化
│   │   ├── markdown_manager.rs              # Markdown 文件处理
│   │   ├── plugins_manager.rs               # 插件管理
│   │   ├── budget_manager.rs                # 预算管理
│   │   ├── pricing_manager.rs               # 定价管理
│   │   └── config/                          # 配置文件管理器
│   │       ├── mod.rs
│   │       ├── claude_manager.rs            # Claude 配置读写
│   │       ├── codex_manager.rs             # Codex 配置读写
│   │       ├── gemini_manager.rs            # Gemini 配置读写
│   │       ├── qwen_manager.rs              # Qwen 配置读写
│   │       └── platform_manager.rs          # 平台配置管理
│   │
│   ├── models/                              # 模型层
│   │   ├── mod.rs
│   │   ├── api.rs                           # API 模型
│   │   ├── converter.rs                     # 转换模型
│   │   ├── budget.rs                        # 预算模型
│   │   ├── pricing.rs                       # 定价模型
│   │   ├── stats.rs                         # 统计模型
│   │   └── platforms/                       # 平台特定模型
│   │       ├── mod.rs
│   │       ├── codex.rs                     # Codex 数据模型
│   │       ├── gemini.rs                    # Gemini 数据模型
│   │       └── qwen.rs                      # Qwen 数据模型
│   │
│   ├── core/                                # 核心层
│   │   ├── mod.rs
│   │   ├── error.rs                         # 错误类型与处理
│   │   └── executor.rs                      # CCR 命令执行器
│   │
│   └── utils/                               # 工具层
│       ├── mod.rs
│       └── config_reader.rs                 # 通用配置工具
│
├── Cargo.toml                               # Rust 依赖
├── logs/                                    # 日志文件(自动创建)
└── .gitignore                               # Git 忽略规则
```

### 核心入口点

| 入口文件 | 路径 | 职责 |
|----------|------|------|
| **应用入口** | `/src/main.rs` | 启动 Axum 服务器、初始化日志、创建路由 |
| **错误定义** | `/src/core/error.rs` | 自定义错误类型 |
| **命令执行器** | `/src/core/executor.rs` | CCR CLI 子进程执行 |
| **API 处理器** | `/src/api/handlers/*.rs` | HTTP 请求处理 |
| **配置管理器** | `/src/managers/config/*.rs` | 配置文件读写 |

---

## 项目业务模块

### 1. CCR 配置管理 (10 端点)

**Handler**: `handlers/config.rs`

**功能**:
- 列出所有 CCR 配置
- 切换活跃配置
- 创建/更新/删除配置段
- 验证配置
- 导入/导出配置
- 清理旧备份
- 查看操作历史

**API 端点**:
```
GET    /api/configs              - 列出所有配置
POST   /api/switch               - 切换配置
POST   /api/configs              - 创建配置段
PUT    /api/configs/:name        - 更新配置段
DELETE /api/configs/:name        - 删除配置段
GET    /api/validate             - 验证配置
POST   /api/export               - 导出配置
POST   /api/import               - 导入配置
POST   /api/clean                - 清理备份
GET    /api/history              - 操作历史
```

### 2. 命令执行 (3 端点)

**Handler**: `handlers/command.rs`

**功能**:
- 执行 CCR CLI 命令
- 列出可用命令
- 获取命令帮助

**API 端点**:
```
POST /api/command/execute         - 执行命令
GET  /api/command/list            - 列出命令
GET  /api/command/help/:command   - 命令帮助
```

### 3. Claude Code 管理 (33 端点)

**Handlers**: `handlers/mcp.rs`, `handlers/agents.rs`, `handlers/slash_commands.rs`, `handlers/plugins.rs`

**功能**:
- **MCP 服务器** (5): 列表、添加、更新、删除、启用/禁用
- **Agents** (5): 列表、添加、更新、删除、启用/禁用
- **斜杠命令** (5): 列表、添加、更新、删除、启用/禁用
- **插件** (5): 列表、添加、更新、删除、启用/禁用
- **同步** (17): 基础同步 (5) + 多文件夹管理 (6) + 文件夹特定操作 (3) + 批量操作 (3)

### 4. Codex 管理 (33 端点)

**Handler**: `handlers/platforms/codex.rs`

**功能**:
- **MCP 服务器** (4): 列表、添加、更新、删除
- **配置文件** (4): 列表、添加、更新、删除
- **基础配置** (2): 获取、更新
- **Agents/斜杠命令/插件**: 各 5 端点

### 5. Gemini CLI / Qwen / iFlow 管理

**Handlers**: `handlers/platforms/gemini.rs`, `handlers/platforms/qwen.rs`, `handlers/platforms/iflow.rs`

**功能**:
- **Gemini**: 28 端点 (MCP, Agents, 斜杠命令, 插件, 配置)
- **Qwen**: 28 端点 (同上)
- **iFlow**: 5 端点 (stub 实现)

### 6. 系统信息与版本 (4 端点)

**Handlers**: `handlers/system.rs`, `handlers/version.rs`

**功能**:
- 获取系统信息 (CPU, 内存, OS)
- 获取 CCR 版本
- 检查更新
- 执行更新

### 7. 统计与预算管理 (新增)

**Handlers**: `handlers/stats.rs`, `handlers/budget.rs`, `handlers/pricing.rs`

**功能**:
- 使用统计查看
- 预算管理
- 定价策略管理

---

## 项目代码风格与规范

### 命名约定

#### Rust 命名规范
- **模块名**: `snake_case` (如 `config_manager`, `slash_commands`)
- **类型名**: `PascalCase` (如 `McpServer`, `SystemInfo`)
- **函数名**: `snake_case` (如 `list_mcp_servers`, `execute_command`)
- **常量**: `SCREAMING_SNAKE_CASE` (如 `DEFAULT_PORT`, `MAX_RETRIES`)

#### 文件命名
- **Handler 文件**: 功能名称 (如 `config.rs`, `command.rs`)
- **Manager 文件**: `*_manager.rs` (如 `settings_manager.rs`)
- **Model 文件**: 实体名称 (如 `api.rs`, `converter.rs`)

### 代码风格

#### Rust 代码结构

推荐模块结构:
```rust
// 1. Imports
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use crate::models::api::McpServer;
use crate::managers::config::claude_manager;
use crate::core::error::AppError;

// 2. Type definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMcpRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

// 3. Public functions
pub async fn list_mcp_servers() -> Result<Json<Vec<McpServer>>, AppError> {
    let servers = claude_manager::read_mcp_servers()?;
    Ok(Json(servers))
}

pub async fn create_mcp_server(
    Json(payload): Json<CreateMcpRequest>,
) -> Result<Json<McpServer>, AppError> {
    // 实现逻辑
    Ok(Json(server))
}

// 4. Private helper functions
fn validate_mcp_config(config: &McpServer) -> Result<(), AppError> {
    // 验证逻辑
    Ok(())
}
```

#### Import 规则

按以下顺序分组导入:
```rust
// 1. 标准库
use std::path::PathBuf;
use std::collections::HashMap;

// 2. 外部 crate
use axum::{Router, Json, extract::Path};
use serde::{Deserialize, Serialize};
use tokio::fs;

// 3. 内部模块 (按层级)
use crate::models::api::McpServer;
use crate::managers::config::claude_manager;
use crate::core::error::AppError;
use crate::utils::config_reader;
```

#### 异常处理

使用 `Result` 类型与自定义错误:
```rust
use crate::core::error::AppError;

pub async fn read_config() -> Result<Config, AppError> {
    let content = tokio::fs::read_to_string("config.json")
        .await
        .map_err(|e| AppError::FileReadError(e.to_string()))?;

    let config: Config = serde_json::from_str(&content)
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    Ok(config)
}
```

#### 日志规范

使用 `tracing` 进行结构化日志:
```rust
use tracing::{info, warn, error, debug};

pub async fn process_request(id: &str) -> Result<()> {
    info!(request_id = %id, "Processing request");

    match do_work(id).await {
        Ok(result) => {
            info!(request_id = %id, result = ?result, "Request completed");
            Ok(result)
        }
        Err(e) => {
            error!(request_id = %id, error = %e, "Request failed");
            Err(e)
        }
    }
}
```

#### 参数校验

在 Handler 层验证输入:
```rust
#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub command: String,
}

impl CreateServerRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.is_empty() {
            return Err(AppError::ValidationError("名称不能为空".to_string()));
        }

        if self.name.len() < 3 {
            return Err(AppError::ValidationError("名称至少3个字符".to_string()));
        }

        if self.command.is_empty() {
            return Err(AppError::ValidationError("命令不能为空".to_string()));
        }

        Ok(())
    }
}
```

### 其他规范

- **文档注释**: 使用 `///` 为公开 API 添加文档
- **错误处理**: 使用 `?` 操作符传播错误
- **原子操作**: 文件写入使用临时文件 + 原子重命名
- **并发安全**: 使用 Tokio 的异步 I/O
- **代码格式化**: 使用 `cargo fmt`
- **代码检查**: 通过 `cargo clippy` 无警告

---

## 测试与质量

### 单元测试

(当前未配置,可扩展)

**推荐方式**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_mcp_servers() {
        // 测试逻辑
    }
}
```

### 集成测试

(当前未配置,可扩展)

### 代码质量检查

#### Cargo 检查

```bash
# 编译检查
cargo check

# Clippy 检查
cargo clippy --all-targets --all-features

# 格式化检查
cargo fmt --check

# 构建
cargo build

# 发布构建
cargo build --release
```

### 质量目标

- ✅ **零编译错误**: 所有代码通过 `cargo check`
- ✅ **零 Clippy 警告**: 代码符合 Clippy 规则
- ✅ **代码格式化**: 使用 `cargo fmt`
- 🚧 **单元测试覆盖率**: (待配置) 目标 80%+
- 🚧 **集成测试**: (待配置) 覆盖关键 API 端点

---

## 项目构建、测试与运行

### 环境与配置

#### 环境要求

- **Rust**: 1.85+ (Edition 2024)
- **Cargo**: 最新稳定版

#### 环境变量

```bash
# 日志级别
RUST_LOG=debug              # trace | debug | info | warn | error

# 服务器配置
HOST=127.0.0.1              # 绑定地址
PORT=8081                   # 监听端口
```

### 开发命令

```bash
# 安装依赖(自动)
cargo build

# 启动开发服务器
cargo run

# 自定义端口
cargo run -- --port 8082

# 启用调试日志
RUST_LOG=debug cargo run

# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 发布构建
cargo build --release

# 运行测试
cargo test
```

### 构建流程

**开发模式**:
```bash
cd ccr-ui/backend
cargo run

# 服务器启动在 127.0.0.1:8081
# 日志输出到 logs/ 目录
```

**生产构建**:
```bash
cargo build --release

# 二进制文件:
# target/release/ccr-ui-backend

# 运行:
./target/release/ccr-ui-backend --port 8081
```

### 部署指南

#### 本地部署

```bash
# 构建发布版本
cargo build --release

# 运行
./target/release/ccr-ui-backend --host 0.0.0.0 --port 8081
```

#### 生产部署

1. **构建二进制**:
   ```bash
   cargo build --release
   ```

2. **配置 systemd 服务** (Linux):
   ```ini
   [Unit]
   Description=CCR UI Backend
   After=network.target

   [Service]
   Type=simple
   User=ccr
   WorkingDirectory=/opt/ccr-ui/backend
   ExecStart=/opt/ccr-ui/backend/ccr-ui-backend --port 8081
   Restart=always

   [Install]
   WantedBy=multi-user.target
   ```

3. **配置反向代理** (Nginx):
   ```nginx
   location /api {
       proxy_pass http://127.0.0.1:8081;
       proxy_set_header Host $host;
       proxy_set_header X-Real-IP $remote_addr;
       proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
   }
   ```

---

## Git 工作流程

### 分支策略

- **main**: 主分支,生产环境代码
- **dev**: 开发分支,测试环境代码
- **feature/***: 功能分支
- **bugfix/***: Bug 修复分支

### 提交规范

遵循 Conventional Commits 规范:

```bash
# 功能开发
git commit -m "feat(后端): 添加预算管理 API"

# Bug 修复
git commit -m "fix(后端): 修复配置文件解析错误"

# 重构
git commit -m "refactor(后端): 重构为分层架构"

# 性能优化
git commit -m "perf(后端): 优化配置文件读取性能"

# 文档更新
git commit -m "docs(后端): 更新 API 文档"

# 测试
git commit -m "test(后端): 添加 MCP 端点集成测试"
```

---

## 文档目录(重要)

### 文档存储规范

- **模块文档**: `/ccr-ui/backend/CLAUDE.md` (本文件)
- **上级文档**: `/ccr-ui/CLAUDE.md` (CCR UI 总览)
- **根文档**: `/CLAUDE.md` (项目总览)
- **前端文档**: `/ccr-ui/frontend/CLAUDE.md` (前端模块)

### 相关文件列表

#### 源代码
- `/ccr-ui/backend/src/main.rs` - 入口点与路由
- `/ccr-ui/backend/src/api/` - API 层 (16+ handler 文件)
- `/ccr-ui/backend/src/services/` - 服务层 (2 文件)
- `/ccr-ui/backend/src/managers/` - 管理层 (8+ 文件)
- `/ccr-ui/backend/src/models/` - 模型层 (6 文件)
- `/ccr-ui/backend/src/core/` - 核心层 (error, executor)
- `/ccr-ui/backend/src/utils/` - 工具层 (config_reader)

#### 配置文件
- `/ccr-ui/backend/Cargo.toml` - Rust 依赖
- `/ccr-ui/backend/.gitignore` - Git 忽略规则

#### 构建输出
- `/ccr-ui/backend/target/` - 构建产物
- `/ccr-ui/backend/logs/` - 日志文件

### 外部链接

- **Axum 文档**: https://docs.rs/axum/
- **Tokio 文档**: https://docs.rs/tokio/
- **Serde 文档**: https://serde.rs/
- **Tracing 文档**: https://docs.rs/tracing/
- **Rust Book**: https://doc.rust-lang.org/book/

---

## 常见问题(FAQ)

### Q: 后端如何与 CCR CLI 通信?

A: 后端使用 `executor` 模块将 CCR 作为子进程生成并捕获其输出。这确保后端不重复 CCR 的逻辑,始终使用规范实现。

### Q: 如何安全地修改配置文件?

A: 所有配置管理器使用原子文件操作:
1. 读取现有配置
2. 在内存中解析和修改
3. 写入临时文件
4. 原子重命名到目标文件

即使进程崩溃,这也能防止损坏。

### Q: 如果找不到 CCR 二进制会怎样?

A: 服务器记录警告但继续启动。需要 CCR 的 API 端点将返回错误响应。这允许服务器在 CCR 可能稍后安装的环境中运行。

### Q: 如何为生产环境启用 CORS?

A: 默认情况下,CORS 允许所有来源 (`Any`)。对于生产环境,修改 `main.rs` 中的 CORS 层:
```rust
CorsLayer::new()
    .allow_origin("https://yourdomain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
```

### Q: 日志如何管理?

A: 日志写入 `logs/` 目录,每日轮换。`tracing-appender` crate 自动处理轮换。默认保留 7 天的旧日志。

### Q: 可以独立运行后端吗?

A: 可以!后端完全独立。您可以运行它而无需前端,用于仅 API 使用或与自定义前端集成。

### Q: 如何添加对新平台的支持?

A:
1. 在 `src/models/platforms/<platform>.rs` 创建模型
2. 在 `src/managers/config/<platform>_manager.rs` 创建配置管理器
3. 在 `src/api/handlers/platforms/<platform>.rs` 创建处理器
4. 在 `src/main.rs` 添加路由
5. 如需要,更新转换器

---

**本小姐精心整理的后端模块文档完成啦！分层架构清晰明了,这才是专业的做法呢～(￣▽￣)／**
