# CCR UI 模块指导文件

[根目录](../CLAUDE.md) > **ccr-ui**

---

## 项目架构

### 模块职责

CCR UI 是基于 **Tauri v2** 的原生桌面应用，为多个 AI CLI 工具提供可视化管理界面。

**核心组成**:
1. **Tauri Backend** (`src-tauri/`) - Rust 原生后端；IPC 子模块见 `src-tauri/src/commands/mod.rs`，命令清单见生成文档 `docs/reference/tauri-command-inventory.md`（英：`docs/en/reference/tauri-command-inventory.md`）
2. **Frontend** (`src/`) - React 19 + TanStack Query 单页应用（视觉规则见本目录 `AGENTS.md` 与 `DESIGN.md` 行情终端）
3. **ccr-db** (`../crates/ccr-db/`) - 独立 workspace crate，提供 SQLite 数据库、CheckIn、加密等服务

**支持平台**:
- **Claude Code** - MCP 服务器、Agents、斜杠命令、插件、Settings、Hooks
- **Codex** - Profiles、MCP、Agents、斜杠命令、插件、Auth
- **Gemini CLI** - Settings、MCP、Agents、斜杠命令、插件
- **Droid** - Settings、MCP、Agents、Plugins、Models、Profiles
- **OpenCode** - Settings、Keybindings、Themes、Providers、MCP

### 架构总览

```
ccr-ui/
├── src-tauri/                      # Tauri v2 Rust 后端 (原生嵌入)
│   ├── src/
│   │   ├── main.rs                # 应用入口 (AppState 初始化、后台任务)
│   │   ├── state.rs               # AppState (SQLite 连接池、缓存、环境注册表)
│   │   ├── events.rs              # Tauri Event 系统 (替代 WebSocket)
│   │   ├── commands/              # `pub mod` 见 commands/mod.rs；注册表见 handler_registry.rs
│   │   │                          # 生成清单：docs/reference/tauri-command-inventory.md
│   │   └── platform/              # 执行环境抽象层
│   │       ├── mod.rs             # ExecutionEnvironment trait + EnvironmentRegistry
│   │       ├── local.rs           # 本地环境 (委托 ccr 核心库)
│   │       └── wsl.rs             # WSL 环境 (Windows only)
│   ├── Cargo.toml                 # Tauri 依赖
│   └── tauri.conf.json            # Tauri 配置
│
├── src/                            # React 19 前端 (SPA)
│   ├── main.tsx                   # 挂载 Query + Router
│   ├── shell/                      # 壳层：App、router、routeCatalog、MainLayout
│   ├── features/                   # 域页面（claude/codex/grok/usage/…）
│   ├── api/                        # 域封装 + generated typed IPC；tauri.ts 为兼容门面
│   ├── types/generated/           # ts-rs 生成的 Rust DTO
│   ├── ui/                         # 共享原语
│   ├── styles/                     # tokens.css 等（视觉以 DESIGN.md 为准）
│   └── i18n/
├── package.json                    # packageManager: bun@1.4.0
├── vite.config.ts
└── CLAUDE.md                       # 本文件；视觉规则不在此重定义
```

**设计哲学**:
- **原生嵌入**: 后端编译进 Tauri 二进制，无需独立进程
- **IPC 通信**: 前端通过 `invoke()` 直接调用 Rust 函数，零网络开销
- **事件驱动**: 后端通过 `app_handle.emit()` 推送事件，前端 `listen()` 接收
- **原子操作**: 所有文件写入使用 tempfile + rename 原子操作
- **类型安全**: Rust `#[tauri::command]` + TypeScript `invoke()` 双端类型安全
- **环境抽象**: `ExecutionEnvironment` trait 支持 Local/WSL/SSH 多环境

## Design Context

视觉、品牌与审美以本目录 `AGENTS.md` 与 `DESIGN.md`（行情终端 / market terminal）为准。本文件不重定义色板、token 或设计原则。若下文与这两份文件冲突，以 `AGENTS.md` 和 `DESIGN.md` 为准。浏览器或 UI 工具可用不等于已授权操作界面。

导航见 `./code_map.md`（`src/shell`、`src/features`、`src/api`、generated types）。

---

## 项目技术栈

### Tauri 后端技术栈 (Rust)

| 类别 | 技术 | 用途 |
|------|------|------|
| **框架** | Tauri v2 | 桌面应用框架 |
| **运行时** | Tokio | 异步运行时 |
| **数据库** | SQLite (r2d2 连接池) | 本地数据存储 |
| **序列化** | Serde | JSON/TOML/YAML |
| **核心库** | ccr (workspace) | 配置管理核心 |
| **数据库服务** | ccr-db (workspace) | CheckIn、加密、用量导入 |

### 前端技术栈 (TypeScript/React)

| 类别 | 技术 | 版本 | 用途 |
|------|------|------|------|
| **框架** | React | 19.2.8 | UI 框架 |
| **构建** | Vite | 8.2.2 | 构建工具 |
| **路由** | React Router | 8.3.1 | 路由管理 |
| **状态** | Zustand | 5.0.15 | 客户端状态 |
| **查询** | TanStack React Query | 5.102.8 | 服务端状态 |
| **组件** | Radix UI | 见 package.json `@radix-ui/*` | 无样式原语 |
| **样式** | Tailwind CSS | 4.3.3 | CSS 框架 |
| **IPC** | @tauri-apps/api | 2.11.1 | Tauri invoke() 通信 |
| **类型** | TypeScript | ^6.0.3 | 类型安全 |

---

## 关键设计模式

### IPC 通信模式

前端所有 API 调用通过 `@tauri-apps/api/core` 的 `invoke()` 函数。新增 wrapper 一律放
`src/api/domains/<domain>.ts`（`src/api/tauri.ts` 是冻结的兼容门面，见
`.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`）；已类型化的 domain（usage V2、claude_observer）
返回类型使用 ts-rs 生成绑定（`src/types/generated/`，契约见
`.trellis/spec/ccr/backend/typed-ipc-bindings.md`）：

```typescript
// src/api/domains/stats.ts
import { invoke } from '@tauri-apps/api/core'
import type { UsageSummaryDto } from '../../types/generated/usage/UsageSummaryDto'
export const getUsageSummaryV2 = async (platform?: string): Promise<UsageSummaryDto> =>
  invoke('get_usage_summary_v2', { platform })
```

### 事件推送模式

后端通过 Tauri Event 系统替代 WebSocket：

```rust
// 后端发送事件
app_handle.emit("app-log", log_payload)?;
app_handle.emit("token-stats", stats)?;
```

```typescript
// 前端监听事件
import { listen } from '@tauri-apps/api/event'
const unlisten = await listen<LogMessage>('app-log', (event) => { ... })
```

### 执行环境抽象

```rust
// platform/mod.rs
pub trait ExecutionEnvironment: Send + Sync {
    fn name(&self) -> &str;
    fn env_type(&self) -> &str;  // "local" | "wsl" | "ssh"
    fn list_platforms(&self) -> Result<Vec<String>>;
    fn read_config(&self, platform: &str, path: &str) -> Result<String>;
    fn write_config(&self, platform: &str, path: &str, content: &str) -> Result<()>;
    fn detect_cli_status(&self) -> Result<Vec<CliStatus>>;
}
```

---

## 项目构建、测试与运行

### 环境要求

- **Rust**: 1.88+ (Edition 2024)
- **Node.js**: 18.x+
- **Tauri CLI**: `cargo install tauri-cli`

### 开发命令

仓库约定前端包管理器是 **bun**（见根 `AGENTS.md`）。下列命令在 `ccr-ui/` 下运行：

```bash
cd ccr-ui

bun install

# 网页预览（Playwright/视觉工作默认走这条，不要默认 tauri:dev）
bun run dev:web -- --host 127.0.0.1 --strictPort

# Tauri 开发模式（仅当任务明确需要原生窗口 API）
bun run tauri:dev

# 类型检查 / lint / smoke
bun run type-check
bun run lint
bun run test

# Tauri Rust
bun run tauri:check
```

### Just 命令 (从根目录)

| 命令 | 说明 |
|------|------|
| `just tauri-dev` | 启动 Tauri 开发模式 |
| `just tauri-build` | 构建桌面应用安装包 |
| `just tauri-check` | Tauri Rust 编译检查 |
| `just tauri-clippy` | Tauri Rust Clippy 检查 |
| `just frontend-typecheck` | 前端 TypeScript 检查 |
| `just frontend-lint` | 前端 ESLint 检查 |

---

## 代码规范

### Tauri 命令 (Rust)

- 所有命令函数使用 `#[tauri::command]`
- 文件 I/O 使用 `tokio::task::spawn_blocking`
- 返回 `Result<T, String>` 格式
- 命名: `snake_case` (如 `list_configs`, `switch_config`)

### 前端 (TypeScript/React)

- 组件: React 函数组件（`PascalCase.tsx`），页面放 `src/features/`，壳层放 `src/shell/`
- API 调用: 新业务 wrapper 放 `src/api/domains/*`；`src/api/tauri.ts` 是兼容门面
- 样式: 遵循 `DESIGN.md` / `src/styles/tokens.css`；不要在本文件重定义视觉 token
- 注释: 中文注释

---

## 文档目录

- **模块文档**: `/ccr-ui/CLAUDE.md` (本文件)
- **前端文档**: `/ccr-ui/CLAUDE.md`（本文件）
- **根文档**: `/CLAUDE.md` (项目总览)

---
