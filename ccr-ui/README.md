# CCR UI - Vue 3 + Axum + Tauri 全栈应用

面向 CCR（Claude Code Configuration Switcher）的图形化/桌面化体验。前端 Vue 3 + Vite + Pinia，后端 Axum，支持 Web 模式与 Tauri 桌面模式。版本 3.6.2。

## 功能速览
- 配置管理：查看/切换/验证/历史/备份，覆盖全部 CLI 能力
- 命令执行：可视化运行所有 CCR 命令，实时输出
- WebDAV 多目录同步：目录注册、启用/禁用、单目录与全量 push/pull/status
- 平台与系统：平台概览、系统信息/健康检查
- 运行模式：Web（HTTP API）与桌面（Tauri invoke，自动切换）

## 推荐使用方式（无需手工准备前端）
```bash
ccr ui                  # 优先使用本地源码，其次 ~/.ccr/ccr-ui/，最后自动从 GitHub 下载
# 默认端口：前端 3000，后端 8081，可用 -p/--backend-port 覆盖
```

## 仓库开发快速开始（使用 just）
```bash
cd ccr-ui
just s                 # 启动前后端开发模式（最常用）
just quick-start       # 首次使用：检查依赖 + 安装 + 启动
```
常用快捷：
- `just s` 开发同启（frontend + backend）
- `just i` 安装依赖
- `just b` 构建生产版
- `just c` 检查
- `just t` 测试
- `just f` 格式化
- 更多见 `just --list`

## 先决条件
- Rust 1.85+（workspace 共享依赖）
- Node.js 18+（npm）
- 已安装 `ccr`（PATH 可见）
- 建议安装 `just`：`cargo install just`（或包管理器）

## 架构与工作区
```
ccr/ (workspace root)
|-- Cargo.toml          # 共享依赖
|-- src/                # CCR CLI + lib
|-- ccr-ui/
|   |-- backend/        # Axum 服务（workspace 成员）
|   `-- frontend/       # Vue 3 + Vite + Pinia + Tauri
`-- justfile            # 开发任务
```
后端与主工程共享版本与依赖，推荐在 workspace 根运行构建/测试。

## 手动开发（不依赖 just）
```bash
# 后端
cd ccr-ui/backend
cargo run -- --port 8081

# 前端
cd ../frontend
npm install
npm run dev            # http://localhost:5173
```
前端通过 API 访问 `http://localhost:8081`（默认，依赖环境变量或配置可重写）。

## 生产构建
```bash
cd ccr-ui
just build             # 构建后端 + 前端
just run-prod          # 运行后端并服务前端静态产物
```
或手动：
```bash
cd ccr-ui/backend && cargo build --release
cd ../frontend && npm install && npm run build
```
前端产物位于 `frontend/dist/`，后端可通过参数指定前端目录或使用默认。

## 桌面模式（Tauri 2）
```bash
cd ccr-ui
just tauri-dev         # 桌面开发
just tauri-build       # 打包桌面安装包
```
系统依赖：Linux 需 `libwebkit2gtk-4.0-dev build-essential`，macOS 需 Xcode CLT，Windows 需 VS C++ Build Tools。

## 常用 API（后端）
- 配置：`GET /api/configs`，`POST /api/switch`，`POST /api/validate`，`POST /api/export`，`POST /api/import`，`GET /api/history`
- 命令执行：`POST /api/command/execute`，`GET /api/command/list`，`GET /api/command/help/:command`
- 同步：`GET /api/sync/status`，`POST /api/sync/push`，`POST /api/sync/pull`，目录管理与批量操作同 CLI 语义

## 故障排查
- 后端端口占用：改用 `--port`，或关闭占用进程
- 前端连不上后端：确认 8081 运行，检查浏览器控制台与网络面板
- CLI 调用异常：确认 `ccr` 在 PATH，版本为 3.6.2，必要时加 `CCR_LOG_LEVEL=debug`

## 许可证
MIT（与主项目一致）

---
Last Updated: 2025-01-24 / Version: 3.6.2
