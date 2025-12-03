# CCR UI - Vue 3 + Axum + Tauri 全栈应用

面向 CCR（Claude Code Configuration Switcher）的图形化/桌面化体验。前端 Vue 3 + Vite + Pinia，后端 Axum，支持 Web 模式与 Tauri 桌面模式。版本 3.6.2。

## 功能概览

### 核心功能
- **配置管理**：可视化查看、切换、验证、导入/导出、历史记录、备份管理，覆盖全部 CLI 能力
- **命令执行**：图形化运行所有 CCR 命令，实时查看命令输出
- **多平台支持**：统一管理 Claude Code、Codex、Gemini CLI、Qwen 等 AI 工具配置
- **WebDAV 同步**：多目录同步、注册管理、批量与单目录 push/pull/status
- **格式转换**：不同平台配置间的互相转换（Claude ↔ Codex ↔ Gemini）
- **桌面支持**：Tauri 2.0 构建原生桌面应用

### 支持的 AI 平台
| 平台 | 状态 | 说明 | 配置文件路径 |
|------|------|------|-------------|
| Claude Code | ✅ 完全支持 | Anthropic 官方 CLI | `~/.claude/settings.json` |
| Codex | ✅ 完全支持 | GitHub Copilot CLI | `~/.codex/config.json` |
| Gemini CLI | ✅ 完全支持 | Google Gemini CLI | `~/.gemini/settings.json` |
| Qwen | ✅ 完全支持 | 阿里通义千问 CLI | `~/.qwen/config.json` |
| iFlow | 🚧 基础支持 | iFlow CLI | 开发中 |

## 快速开始

### 推荐方式（使用 CCR CLI）
```bash
ccr ui                  # 自动检测本地源码、用户目录或从 GitHub 下载
ccr ui -p 3000          # 自定义前端端口
ccr ui --backend-port 8081  # 自定义后端端口
# 默认端口：前端 3000，后端 8081
```

CCR CLI 会按以下优先级查找 ccr-ui：
1. **开发环境** - `./ccr-ui/` 或 `../ccr-ui/` (当前工作区)
2. **用户目录** - `~/.ccr/ccr-ui/` (用户安装)
3. **GitHub 下载** - 提示从 GitHub 下载 (首次使用)

### 开发环境（使用 just）
```bash
cd ccr-ui
just quick-start        # 首次使用：检查依赖 → 安装 → 启动
just s                  # 启动前后端开发模式（最常用）
```

**常用 just 命令**：
- `just i` - 安装前后端依赖
- `just b` - 构建生产版本（后端 + 前端）
- `just c` - 代码检查（clippy + 格式检查 + TypeScript 检查）
- `just t` - 运行测试
- `just f` - 格式化代码
- `just s` - 启动开发环境（前端 5173 + 后端 8081）
- `just run-prod` - 运行生产环境
- `just tauri-dev` - Tauri 桌面开发
- `just tauri-build` - 构建桌面安装包
- `just --list` - 查看所有可用命令

## 先决条件

- **Rust 1.85+**（工作区共享依赖）
- **Node.js 18+** + **Bun 1.0+**（包管理器）
- **CCR CLI**（已安装，PATH 可见）
- **just**（可选但推荐：`cargo install just`）
- **Tauri 依赖**（可选，桌面模式）：
  - Linux: `libwebkit2gtk-4.0-dev build-essential`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio C++ Build Tools

## 项目架构

### 工作区结构
```
ccr/ (workspace root)
├── Cargo.toml          # 共享依赖配置
├── src/                # CCR CLI + 核心库
├── ccr-ui/            # CCR UI 全栈应用
│   ├── backend/        # Axum REST API 服务器（129 个端点）
│   ├── frontend/       # Vue 3 + Vite + Pinia 前端
│   └── docs/           # VitePress 文档站点
└── justfile            # 开发任务自动化
```

### 前后端通信
- **前端 → 后端**：Axios HTTP 请求（REST API）
- **后端 → CLI**：直接调用 CCR 命令 + 文件系统操作
- **状态管理**：Pinia stores（客户端状态）
- **API 地址**：默认 `http://localhost:8081`

### 技术栈

**后端（Rust）**
- Web 框架：`axum` 0.7 + `tokio` 异步运行时
- HTTP 中间件：`tower` 0.5 + `tower-http`
- 序列化：`serde` + `serde_json` + `toml`
- 错误处理：`anyhow` + `thiserror`
- 日志：`tracing` + `tracing-subscriber`
- HTTP 客户端：`reqwest`
- 配置管理：`dirs` + `tempfile`

**前端（Vue 3）**
- 框架：`vue` 3.5.22 + `vue-router` 4.4
- 状态管理：`pinia` 2.2.6
- HTTP 客户端：`axios` 1.7.9
- UI 组件：`lucide-vue-next` (图标)
- 样式：`tailwindcss` 3.4.17
- 构建工具：`vite` 7.1.11
- 类型检查：`typescript` 5.7 + `vue-tsc` 2.2

## API 端点详解

### 1. Claude Code API（33 个端点）

**MCP 服务器管理**（`/api/mcp`）
- `GET    /api/mcp` - 列出所有 MCP 服务器
- `POST   /api/mcp` - 添加 MCP 服务器
- `PUT    /api/mcp/:name` - 更新 MCP 服务器
- `DELETE /api/mcp/:name` - 删除 MCP 服务器
- `PUT    /api/mcp/:name/toggle` - 启用/禁用 MCP 服务器

**Agents**（`/api/agents`）
- `GET    /api/agents` - 列出所有 Agents
- `POST   /api/agents` - 添加 Agent
- `PUT    /api/agents/:name` - 更新 Agent
- `DELETE /api/agents/:name` - 删除 Agent
- `PUT    /api/agents/:name/toggle` - 启用/禁用 Agent

**Slash Commands**（`/api/slash-commands`）
- `GET    /api/slash-commands` - 列出所有 Slash 命令
- `POST   /api/slash-commands` - 添加 Slash 命令
- `PUT    /api/slash-commands/:name` - 更新 Slash 命令
- `DELETE /api/slash-commands/:name` - 删除 Slash 命令
- `PUT    /api/slash-commands/:name/toggle` - 启用/禁用 Slash 命令

**Plugins**（`/api/plugins`）
- `GET    /api/plugins` - 列出所有插件
- `POST   /api/plugins` - 添加插件
- `PUT    /api/plugins/:name` - 更新插件
- `DELETE /api/plugins/:name` - 删除插件
- `PUT    /api/plugins/:name/toggle` - 启用/禁用插件

**配置管理**（`/api/config`）
- `GET    /api/config` - 获取 Claude 配置
- `PUT    /api/config` - 更新 Claude 配置

### 2. Codex API（33 个端点）

**前缀：`/api/codex/`**
支持 Profiles、MCP、Agents、Slash Commands、Plugins 管理
- `GET    /api/codex/config` - 获取 Codex 配置
- `PUT    /api/codex/config` - 更新 Codex 配置

### 3. Gemini CLI API（28 个端点）

**前缀：`/api/gemini-cli/`**
支持 MCP、Agents、Slash Commands、Plugins、Config 管理

### 4. Qwen API（28 个端点）

**前缀：`/api/qwen/`**
支持 MCP、Agents、Slash Commands、Plugins、Config 管理

### 5. iFlow API（5 个端点 - 基础支持）
- `GET    /api/iflow/mcp` - 获取 iFlow MCP 服务器
- `POST   /api/iflow/mcp` - 添加/更新 iFlow MCP 服务器
- `GET    /api/iflow/agents` - 获取 iFlow Agents
- `GET    /api/iflow/slash-commands` - 获取 iFlow Slash 命令
- `GET    /api/iflow/plugins` - 获取 iFlow 插件

### 6. 工具类 API
- `POST   /api/converter/convert` - 转换配置文件格式
- `POST   /api/sync/claude-to-codex` - 从 Claude 同步到 Codex
- `POST   /api/command/execute` - 执行 CCR CLI 命令
- `GET    /api/system/info` - 获取系统信息
- `GET    /api/version` - 获取后端版本

### 7. CCR 核心 API
- `GET    /api/configs` - 列出所有配置
- `POST   /api/switch` - 切换配置
- `POST   /api/validate` - 验证配置
- `POST   /api/export` - 导出配置
- `POST   /api/import` - 导入配置
- `GET    /api/history` - 查看历史记录

## 前端路由

### 主路由
```
/                        - 首页/仪表板（平台概览）
/configs                 - CCR 配置管理
/commands                - CCR 命令执行器
/converter               - 配置格式转换
/sync                    - WebDAV 同步管理
```

### Claude Code 路由
```
/claude                  - Claude 概览
/mcp                     - MCP 服务器管理
/agents                  - Agents 管理
/slash-commands          - Slash 命令管理
/plugins                 - 插件管理
```

### Codex 路由
```
/codex                   - Codex 概览
/codex/profiles          - Profiles 管理
/codex/mcp               - MCP 服务器管理
/codex/agents            - Agents 管理
/codex/slash-commands    - Slash 命令管理
/codex/plugins           - 插件管理
```

### Gemini CLI 路由
```
/gemini-cli              - Gemini 概览
/gemini-cli/mcp          - MCP 服务器管理
/gemini-cli/agents       - Agents 管理
/gemini-cli/slash-commands
/gemini-cli/plugins
```

### Qwen 路由
```
/qwen                    - Qwen 概览
/qwen/mcp                - MCP 服务器管理
/qwen/agents             - Agents 管理
/qwen/slash-commands
/qwen/plugins
```

### iFlow 路由（开发中）
```
/iflow                   - iFlow 概览
/iflow/mcp               - MCP 服务器管理
/iflow/agents            - Agents 管理
/iflow/slash-commands
/iflow/plugins
```

## 手动开发（不依赖 just）

### 后端开发
```bash
cd ccr-ui/backend
cargo run -- --port 8081              # 启动开发服务器（默认 8081）
cargo watch -x run                    # 监听文件变更自动重启
RUST_LOG=debug cargo run              # 开启 debug 日志
```

### 前端开发
```bash
cd ccr-ui/frontend
bun install                           # 安装依赖
bun run dev                           # 启动开发服务器（http://localhost:5173）
bun run build                         # 构建生产版本
bun run type-check                    # TypeScript 类型检查
bun run lint                          # ESLint 检查
```

前端通过 API 访问 `http://localhost:8081`（可通过环境变量配置）。

### 环境变量配置

**后端环境变量**
```bash
RUST_LOG=info              # 日志级别：trace, debug, info, warn, error
RUST_BACKTRACE=1          # 启用错误回溯
PORT=8081                 # 服务端口号
```

**前端环境变量**（`.env` 或 `.env.local`）
```bash
VITE_API_BASE_URL=http://localhost:8081    # 后端 API 地址
VITE_LOG_LEVEL=debug                        # 前端日志级别
```

## 生产部署

### 方式 1：使用 just 命令
```bash
cd ccr-ui
just build              # 构建后端 + 前端生产版本
just run-prod           # 运行后端并服务前端静态文件
```

### 方式 2：手动构建
```bash
# 构建后端
cd ccr-ui/backend
cargo build --release
cp target/release/ccr-ui-backend ../dist/

# 构建前端
cd ../frontend
bun install && bun run build
cp -r dist/* ../dist/static/

# 运行
./dist/ccr-ui-backend --port 8081 --static-dir ./dist/static
```

**构建产物**：
- 后端可执行文件：`ccr-ui/backend/target/release/ccr-ui-backend`
- 前端静态文件：`ccr-ui/frontend/dist/`

### Docker 部署（可选）
```dockerfile
FROM rust:1.85 as backend-builder
WORKDIR /app/ccr-ui/backend
COPY . .
RUN cargo build --release

FROM node:18 as frontend-builder
WORKDIR /app/ccr-ui/frontend
COPY frontend .
RUN bun install && bun run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/ccr-ui/backend/target/release/ccr-ui-backend /usr/local/bin/
COPY --from=frontend-builder /app/ccr-ui/frontend/dist /usr/local/share/ccr-ui/static
EXPOSE 8081
CMD ["ccr-ui-backend", "--port", "8081", "--static-dir", "/usr/local/share/ccr-ui/static"]
```

## Tauri 桌面模式

### 开发模式
```bash
cd ccr-ui
just tauri-dev          # 启动 Tauri 开发环境
```

### 构建桌面应用
```bash
just tauri-build        # 构建安装包
# 产物在 src-tauri/target/release/bundle/
```

### 平台特定依赖

**Linux Ubuntu/Debian**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS**
```bash
# 安装 Xcode Command Line Tools
xcode-select --install
```

**Windows**
- 安装 [Visual Studio Community](https://visualstudio.microsoft.com/)
- 勾选 "C++ build tools" 和 Windows 10/11 SDK

## 数据模型

### 后端核心模型（Rust）

**MCP 服务器**（`models/claude.rs`）
```rust
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}
```

**Agents**（`models/claude.rs`）
```rust
pub struct Agent {
    pub name: String,
    pub description: String,
    pub instructions: String,
}
```

**Slash 命令**（`models/claude.rs`）
```rust
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub command: String,
}
```

**插件**（`models/claude.rs`）
```rust
pub struct Plugin {
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}
```

**Codex Profile**（`models/codex.rs`）
```rust
pub struct CodexProfile {
    pub name: String,
    pub description: String,
    pub settings: serde_json::Value,
}
```

## 测试

### 后端测试
```bash
cd ccr-ui/backend
cargo test              # 运行所有测试
cargo test --lib       # 仅运行单元测试
cargo test -- --nocapture  # 显示测试输出
```

### 前端测试
```bash
cd ccr-ui/frontend
bun run type-check     # TypeScript 类型检查
bun run lint           # ESLint 检查
bun test              # 运行测试（如果配置了测试框架）
```

### 端到端测试（可选）
可以使用 Playwright 或 Cypress 进行端到端测试：
```bash
bun add -d playwright
npx playwright test    # 运行 E2E 测试
```

## 故障排查

### 常见问题

**1. 后端端口被占用**
```bash
# 解决方法 1：使用不同端口
cargo run -- --port 9090

# 解决方法 2：查找并终止占用进程
sudo lsof -i :8081
kill -9 <PID>
```

**2. 前端无法连接后端**
- 确认后端运行在 `http://localhost:8081`
- 检查浏览器控制台网络（Network）面板
- 确认 CORS 配置已启用
- 检查防火墙设置

**3. CLI 调用异常**
- 确认 `ccr` 已在 PATH 中
- 检查版本：`ccr --version`（应为 3.6.2+）
- 开启调试日志：`CCR_LOG_LEVEL=debug ccr ui`
- 检查权限：`chmod +x ~/.ccr/ccr-ui/backend/target/release/ccr-ui-backend`

**4. Node.js 或 npm 相关错误**
- 确认 Node.js 版本：`node --version`（需 ≥ 18）
- 确认 Bun 版本：`bun --version`（需 ≥ 1.0）
- 删除 node_modules 重新安装：`rm -rf node_modules && bun install`

**5. Tauri 构建失败**
- Linux：确认已安装 `libwebkit2gtk-4.0-dev`
- macOS：确认已安装 Xcode CLT
- Windows：确认已安装 Visual Studio C++ Build Tools

**6. CORS 错误**
后端默认允许所有来源，如需限制：
```rust
// backend/src/main.rs
.layer(
    CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
)
```

## 配置文件路径

### AI 工具配置
- **Claude Code**: `~/.claude/settings.json`
- **Codex**: `~/.codex/config.json`
- **Gemini CLI**: `~/.gemini/settings.json`
- **Qwen**: `~/.qwen/config.json`

### CCR UI 相关
- **日志**: `~/.ccr/logs/` 或 `./ccr-ui/logs/`
- **后端日志**: `~/.ccr/logs/ccr-ui-backend.log`
- **前端构建产物**: `ccr-ui/frontend/dist/`
- **Tauri 配置**: `ccr-ui/src-tauri/tauri.conf.json`

## FAQ

### Q: ccr-ui 如何与后端通信？
A: Vue 前端使用 Axios 发送 REST API 请求到 Axum 后端（端口 8081）。所有状态通过 Pinia stores 在客户端管理。

### Q: 可以自定义前后端端口吗？
A: 可以！
```bash
ccr ui -p 3000 --backend-port 8081    # 前端 3000，后端 8081
```

### Q: 配置文件存放在哪里？
A:
- Claude Code: `~/.claude/settings.json`
- Codex: `~/.codex/config.json`
- Gemini: `~/.gemini/settings.json`
- Qwen: `~/.qwen/config.json`

### Q: 如何添加新的 CLI 工具支持？
A:
1. 在 `backend/src/config/` 添加配置读取器
2. 在 `backend/src/models/` 添加数据模型
3. 在 `backend/src/handlers/` 添加 API 处理器
4. 在 `backend/src/main.rs` 添加路由
5. 在 `frontend/src/views/` 添加前端视图
6. 在 `frontend/src/router/` 更新路由配置

### Q: 什么是 liquid glass 设计风格？
A: 现代毛玻璃（glassmorphism）设计风格，特点包括：
- 毛玻璃效果（backdrop-filter: blur）
- 微妙的渐变和阴影
- 平滑的过渡动画
- 支持亮色/暗色主题

### Q: 如何部署到生产环境？
A:
```bash
cd ccr-ui
just b                   # 构建前端
cargo build --release    # 构建后端
# 复制产物到服务器并运行
```

### Q: Web 模式和 Tauri 模式有什么区别？
A:
- **Web 模式**：运行在浏览器中，通过 HTTP 访问
- **Tauri 模式**：原生桌面应用，使用系统 Webview，性能更好，可访问系统 API

### Q: 支持哪些浏览器？
A:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- 不支持 IE

### Q: 如何贡献代码？
A:
1. Fork 仓库
2. 创建特性分支：`git checkout -b feature/my-feature`
3. 提交更改：`git commit -am '添加新功能'`
4. 推送到分支：`git push origin feature/my-feature`
5. 创建 Pull Request

### Q: 如何调试后端 API？
A:
- 使用 `cargo run -- --port 8081` 启动后端
- 访问 `http://localhost:8081/api/version` 验证
- 使用 Postman 或 curl 测试 API
- 查看日志：`tail -f ~/.ccr/logs/ccr-ui-backend.log`

### Q: 支持多用户吗？
A: 目前 ccr-ui 是单用户应用，每个用户使用自己的配置目录（`~/.claude/` 等）。

## 性能优化建议

### 后端优化
- 使用 `cargo build --release` 构建生产版本
- 启用 Rust 的 LTO（链接时优化）
- 合理配置 tokio 线程池大小

### 前端优化
- 使用 `bun run build` 构建生产版本（自动启用 Tree Shaking）
- 配置 CDN 加速静态资源
- 启用 Gzip/Brotli 压缩
- 使用 HTTP/2 或 HTTP/3

## 安全建议

### 后端安全
- 生产环境启用 HTTPS（通过 Nginx 反向代理）
- 限制 API 访问来源
- 敏感操作添加认证中间件
- 定期更新依赖包：
  ```bash
  cd backend && cargo update
  cd frontend && bun update
  ```

### 前端安全
- 验证所有用户输入
- 防止 XSS 攻击（Vue 默认转义内容）
- 配置 CSP（Content Security Policy）
- 不暴露敏感信息在日志中

## 贡献指南

### 代码规范
- **Rust**: 遵循 rustfmt 和 clippy 建议
- **Vue**: 使用 Composition API，`<script setup>` 语法
- **TypeScript**: 启用严格模式
- **Git Commit**: 遵循 Conventional Commits

### 开发流程
1. 从 `dev` 分支创建特性分支
2. 编写代码并添加测试
3. 运行测试：`just t`
4. 代码检查：`just c`
5. 提交 PR 到 `dev` 分支

## 许可证

MIT License（与主项目一致）

---

**最后更新**: 2025-01-24
**版本**: 3.6.2
**文档状态**: ✅ 完善版

## 相关链接

- [CCR 主项目](../README.md)
- [CCR 文档](docs/)
- [GitHub 仓库](https://github.com/bahayonghang/ccr)
- [问题反馈](https://github.com/bahayonghang/ccr/issues)
