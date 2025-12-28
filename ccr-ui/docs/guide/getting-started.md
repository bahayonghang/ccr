# 快速开始

面向 v3.9.4 的 CCR UI 使用与本地开发指南。

## 先决条件
- Rust 1.85+（含 Cargo）
- Node.js 18+（含 npm）
- 已安装 `ccr`（PATH 可见）
- 建议：`just`（`cargo install just`）

## 最快捷的方式：直接用 CLI
```bash
ccr ui
```
自动检测顺序：当前/父目录源码 → `~/.ccr/ccr-ui/` → GitHub 下载。默认端口：前端 3000，后端 8081，可用 `-p/--backend-port` 覆盖。

## 仓库开发
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr/ccr-ui
just s             # 一键启动前后端开发（最常用）
```
首次可用 `just quick-start`（检查依赖 + 安装 + 启动）。

常用 just：
- `just s` / `just quick-start`
- `just i`（安装）`just b`（构建）`just c`（检查）`just t`（测试）`just f`（格式化）
- 查看全部：`just --list`

## 手动运行（不依赖 just）
后端：
```bash
cd backend
cargo run -- --port 8081
```
前端：
```bash
cd frontend
npm install
npm run dev   # http://localhost:5173
```

## 验证
- 后端健康检查：`http://127.0.0.1:8081/api/system/info`
- 前端访问：`http://localhost:5173`（或 `ccr ui` 设定的端口）

## 生产构建
```bash
cd ccr-ui
just build        # 构建后端 + 前端
just run-prod     # 使用构建产物运行
```
或手动：
```bash
cd backend && cargo build --release
cd ../frontend && npm install && npm run build
```

## 桌面（Tauri 2）
```bash
cd ccr-ui
just tauri-dev
just tauri-build
```
系统依赖：Linux 需 `libwebkit2gtk-4.0-dev build-essential`，macOS 需 Xcode CLT，Windows 需 VS C++ Build Tools。
