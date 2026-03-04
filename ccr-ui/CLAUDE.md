# CCR UI 模块指导文件

[根目录](../CLAUDE.md) > **ccr-ui**

---

## 项目架构

### 模块职责

CCR UI 是基于 **Tauri v2** 的原生桌面应用，为多个 AI CLI 工具提供可视化管理界面。

**核心组成**:
1. **Tauri Backend** (`frontend/src-tauri/`) - Rust 原生后端，通过 `#[tauri::command]` IPC 提供 141+ 命令
2. **Frontend** (`frontend/`) - Vue.js 3 单页应用 (Liquid Glass 设计)
3. **ccr-db** (`../ccr-db/`) - 独立 workspace crate，提供 SQLite 数据库、CheckIn、加密等服务

**支持平台**:
- **Claude Code** - MCP 服务器、Agents、斜杠命令、插件、Settings、Hooks
- **Codex** - Profiles、MCP、Agents、斜杠命令、插件、Auth
- **Gemini CLI** - Settings、MCP、Agents、斜杠命令、插件
- **Qwen** - Settings、MCP、Agents、斜杠命令、插件
- **iFlow** - Settings、MCP、斜杠命令
- **Droid** - Settings、MCP、Agents、Plugins、Models、Profiles
- **OpenCode** - Settings、Keybindings、Themes、Providers、MCP

### 架构总览

```
ccr-ui/
├── frontend/
│   ├── src-tauri/                  # Tauri v2 Rust 后端 (原生嵌入)
│   │   ├── src/
│   │   │   ├── main.rs            # 应用入口 (AppState 初始化、后台任务)
│   │   │   ├── state.rs           # AppState (SQLite 连接池、缓存、环境注册表)
│   │   │   ├── events.rs          # Tauri Event 系统 (替代 WebSocket)
│   │   │   ├── commands/          # 141+ Tauri IPC 命令 (13 个子模块)
│   │   │   │   ├── config.rs      # 配置管理
│   │   │   │   ├── claude.rs      # Claude Code 平台
│   │   │   │   ├── codex.rs       # Codex 平台
│   │   │   │   ├── gemini.rs      # Gemini 平台
│   │   │   │   ├── qwen.rs        # Qwen 平台
│   │   │   │   ├── iflow.rs       # iFlow 平台
│   │   │   │   ├── droid.rs       # Droid 平台
│   │   │   │   ├── opencode.rs    # OpenCode 平台
│   │   │   │   ├── checkin.rs     # CheckIn 签到
│   │   │   │   ├── stats.rs       # 统计与费用
│   │   │   │   ├── sync.rs        # WebDAV 同步
│   │   │   │   ├── system.rs      # 系统信息
│   │   │   │   ├── converter.rs   # 配置转换
│   │   │   │   ├── ui_state.rs    # UI 收藏/历史
│   │   │   │   ├── waf.rs         # WAF WebView Bypass
│   │   │   │   ├── unified_mcp.rs # 跨平台 MCP 管理
│   │   │   │   ├── environment.rs # 执行环境管理
│   │   │   │   └── wsl.rs         # WSL 管理 (Windows only)
│   │   │   └── platform/          # 执行环境抽象层
│   │   │       ├── mod.rs         # ExecutionEnvironment trait + EnvironmentRegistry
│   │   │       ├── local.rs       # 本地环境 (委托 ccr 核心库)
│   │   │       └── wsl.rs         # WSL 环境 (Windows only)
│   │   ├── Cargo.toml             # Tauri 依赖
│   │   └── tauri.conf.json        # Tauri 配置
│   │
│   ├── src/                        # Vue.js 3 前端 (SPA)
│   │   ├── views/                  # 40+ 页面组件
│   │   ├── components/             # 20+ 可复用组件
│   │   ├── composables/            # Vue Composables
│   │   ├── stores/                 # Pinia 状态管理
│   │   ├── api/                    # Tauri invoke() 封装
│   │   │   ├── tauri.ts            # 141+ invoke() 包装函数
│   │   │   └── index.ts            # 统一导出
│   │   ├── router/                 # Vue Router
│   │   ├── types/                  # TypeScript 类型
│   │   └── styles/                 # 全局样式
│   ├── package.json
│   └── vite.config.ts
│
├── docs/                           # VitePress 文档站点
└── CLAUDE.md                       # 本文件
```

**设计哲学**:
- **原生嵌入**: 后端编译进 Tauri 二进制，无需独立进程
- **IPC 通信**: 前端通过 `invoke()` 直接调用 Rust 函数，零网络开销
- **事件驱动**: 后端通过 `app_handle.emit()` 推送事件，前端 `listen()` 接收
- **原子操作**: 所有文件写入使用 tempfile + rename 原子操作
- **类型安全**: Rust `#[tauri::command]` + TypeScript `invoke()` 双端类型安全
- **环境抽象**: `ExecutionEnvironment` trait 支持 Local/WSL/SSH 多环境

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

### 前端技术栈 (TypeScript/Vue)

| 类别 | 技术 | 版本 | 用途 |
|------|------|------|------|
| **框架** | Vue.js | 3.5.22 | UI 框架 |
| **构建** | Vite | 7.1.11 | 构建工具 |
| **路由** | Vue Router | 4.4 | 路由管理 |
| **状态** | Pinia | 2.2.6 | 状态管理 |
| **样式** | Tailwind CSS | 3.4.17 | CSS 框架 |
| **IPC** | @tauri-apps/api | 2.x | Tauri invoke() 通信 |
| **类型** | TypeScript | 5.7.3 | 类型安全 |

---

## 关键设计模式

### IPC 通信模式

前端所有 API 调用通过 `@tauri-apps/api/core` 的 `invoke()` 函数：

```typescript
// frontend/src/api/tauri.ts
import { invoke } from '@tauri-apps/api/core'
export const listConfigs = async () => invoke('list_configs')
export const switchConfig = async (name: string) => invoke('switch_config', { name })
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

```bash
cd ccr-ui/frontend

# 安装前端依赖
npm install

# Tauri 开发模式 (启动桌面应用 + 热重载)
npm run tauri dev
# 或
cargo tauri dev

# 前端独立开发 (仅 Web 预览，无 Tauri IPC)
npm run dev

# 类型检查
npm run type-check

# Lint 检查
npm run lint

# Tauri Rust 编译检查
cd src-tauri && cargo check

# 生产构建 (打包桌面应用)
npm run tauri build
# 或
cargo tauri build
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

### 前端 (TypeScript/Vue)

- 组件: `<script setup lang="ts">` Composition API
- API 调用: 统一通过 `@/api` 导入
- 样式: Tailwind CSS 优先
- 注释: 中文注释

---

## 文档目录

- **模块文档**: `/ccr-ui/CLAUDE.md` (本文件)
- **前端文档**: `/ccr-ui/frontend/CLAUDE.md`
- **根文档**: `/CLAUDE.md` (项目总览)

---
