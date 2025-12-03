# CCR UI - Frontend (Vue.js 3 + Tauri 2.0)

> AI CLI Configuration Manager - Modern Web & Desktop Application

基于 Vue.js 3 和 Tauri 2.0 构建的现代化 AI CLI 配置管理工具，同时支持 Web 和桌面应用。

[![Version](https://img.shields.io/badge/version-2.5.0-blue.svg)](./CHANGELOG.md)
[![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D.svg)](https://vuejs.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131.svg)](https://tauri.app/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6.svg)](https://www.typescriptlang.org/)

## ✨ 特性

### 🎨 设计
- **液态玻璃设计**: 现代化的 Glassmorphism UI
- **响应式布局**: 完美支持桌面端和移动端
- **双主题系统**: 亮色/暗色主题无缝切换
- **流畅动画**: 优雅的过渡和交互效果

### 🚀 双模式运行
- **Web 模式**: 浏览器访问，通过 HTTP API 通信
- **Desktop 模式**: 原生桌面应用，Tauri invoke 零延迟

### 🔧 技术特性
- **模块化架构**: 组件化开发，易于维护
- **TypeScript**: 完整的类型安全
- **统一 API**: 自动检测环境，透明切换后端
- **性能优化**: Desktop 模式性能提升 50x

## 🚀 快速开始

### 环境要求

#### Web 开发
- Node.js >= 18.0.0
- Bun >= 1.0.0

#### Desktop 开发
- Node.js >= 18.0.0
- Bun >= 1.0.0
- Rust >= 1.70
- 系统依赖（根据平台）
  - Linux: `libwebkit2gtk-4.0-dev`, `build-essential`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio C++ Build Tools

### 安装依赖

```bash
# 克隆仓库（如果还没有）
git clone https://github.com/bahayonghang/ccr.git
cd ccr/ccr-ui/frontend

# 安装依赖
bun install
```

### 运行模式

#### 🌐 Web 模式

```bash
# 开发服务器
bun run dev
# 或
bun run dev:web

# 访问 http://localhost:5173
```

#### 🖥️ Desktop 模式

```bash
# Tauri 开发模式（首次启动需编译 Rust，约 5-10 分钟）
bun run tauri:dev

# 或使用 justfile（推荐）
just dev
```

### 构建

#### Web 构建

```bash
bun run build:web
# 产物在 dist/ 目录
```

#### Desktop 构建

```bash
bun run build:desktop
# 或
just build

# 产物在 src-tauri/target/release/bundle/
```

## 📦 使用 Just 命令（推荐）

[Just](https://github.com/casey/just) 提供更简洁的命令：

```bash
# 安装 Just
cargo install just

# 查看所有命令
just

# 常用命令
just dev              # 启动 Tauri 开发模式
just dev-web          # 启动 Web 开发模式
just build            # 构建桌面应用
just build-web        # 构建 Web 版本
just check-all        # 全面代码检查
just clean            # 清理构建产物
```

完整命令列表请查看 [justfile](./justfile)。

### 🌟 使用根目录 justfile（更多功能）

在 `ccr-ui/` 根目录，我们提供了统一的 justfile，包含完整的 Tauri 支持：

```bash
cd ..  # 回到 ccr-ui 根目录

# Tauri 桌面应用命令
just tauri-dev         # 启动 Tauri 开发模式
just tauri-build       # 构建生产版本
just tauri-build-debug # 构建调试版本（更快）
just tauri-check       # 检查 Tauri 环境
just tauri-check-all   # 完整检查（TypeScript + Rust）
just tauri-clippy      # Rust linter
just tauri-fmt         # 格式化 Rust 代码
just tauri-test        # 运行测试
just tauri-clean       # 清理构建产物

# Web 开发命令
just dev               # 启动 Web 开发（后端 + 前端）
just build             # 构建 Web 生产版本
```

**推荐使用根目录 justfile** 的原因：
- ✅ 统一管理 Web 和 Desktop 命令
- ✅ 完整的跨平台支持（Linux/macOS/Windows）
- ✅ 更多实用命令（check、test、fmt、clean 等）

## 🏗️ 项目结构

```
ccr-ui/frontend/
├── src/                        # Vue.js 源码
│   ├── api/                    # API 客户端层
│   │   ├── client.ts           # HTTP API（Web 模式）
│   │   ├── tauri.ts            # Tauri API（Desktop 模式）
│   │   └── index.ts            # 统一入口，自动环境检测
│   ├── components/             # Vue 组件
│   │   ├── EnvironmentBadge.vue  # 环境指示器
│   │   └── ...
│   ├── views/                  # 页面视图
│   ├── router/                 # Vue Router 配置
│   ├── store/                  # Pinia 状态管理
│   ├── types/                  # TypeScript 类型
│   ├── styles/                 # 全局样式
│   ├── App.vue                 # 根组件
│   └── main.ts                 # 入口文件
├── src-tauri/                  # Tauri Rust 后端
│   ├── src/
│   │   └── main.rs             # Rust 主程序（13 个命令）
│   ├── icons/                  # 应用图标
│   ├── Cargo.toml              # Rust 依赖
│   ├── tauri.conf.json         # Tauri 配置
│   └── build.rs                # 构建脚本
├── public/                     # 静态资源
├── dist/                       # Web 构建输出
├── justfile                    # Just 命令定义
├── package.json                # npm 配置
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
├── README.md                   # 本文档
├── README.dev.md               # 开发文档
└── CHANGELOG.md                # 版本历史
```

## 🛠️ 技术栈

### 前端
- **框架**: Vue.js 3.5 (Composition API, `<script setup>`)
- **构建**: Vite 7.1
- **语言**: TypeScript 5.7
- **路由**: Vue Router 4.4
- **状态**: Pinia 2.2
- **样式**: Tailwind CSS 3.4
- **图标**: Lucide Vue Next
- **HTTP**: Axios 1.7

### 桌面应用
- **框架**: Tauri 2.0
- **语言**: Rust (Edition 2021)
- **核心库**: CCR (path dependency, 直接集成)
- **异步**: Tokio 1.48
- **序列化**: Serde + Serde JSON
- **日志**: Tracing

## 🎯 核心功能

### 统一 API 层

自动检测运行环境（Web/Desktop），透明切换后端：

```typescript
import { listConfigs } from '@/api'

// 自动选择：
// - Desktop: Tauri invoke → Rust backend → CCR core
// - Web: HTTP request → Axum backend → CCR core
const configs = await listConfigs()
```

### Tauri 命令（13 个）

**配置管理**:
- `list_profiles`: 列出所有配置
- `switch_profile`: 切换配置
- `get_current_profile`: 获取当前配置
- `validate_configs`: 验证配置

**历史记录**:
- `get_history`: 获取历史
- `clear_history`: 清理历史（TODO）

**云同步**:
- `sync_push`: 推送到云端（TODO）
- `sync_pull`: 从云端拉取（TODO）
- `sync_status`: 同步状态（TODO）

**平台管理**:
- `list_platforms`: 列出所有平台
- `switch_platform`: 切换平台
- `get_current_platform`: 获取当前平台

### 环境指示器

`EnvironmentBadge` 组件显示当前运行环境：
- 🖥️ 桌面应用（显示 Tauri 版本）
- 🌐 Web 版本

## 📊 功能模块

- **首页**: 系统概览和模块导航
- **Claude Code**: 配置管理、云同步、MCP 服务器、Agents、插件
- **Codex**: MCP 服务器、Profiles、基础配置
- **Gemini CLI**: 配置管理和工具集成
- **Qwen**: 阿里通义千问配置管理
- **iFlow**: 工作流配置管理
- **命令中心**: 统一的 CLI 命令执行界面
- **配置转换器**: 跨平台配置格式转换
- **云同步**: WebDAV 云端配置同步

## 🎨 设计风格

### 液态玻璃设计 (Liquid Glass)

- **背景**: 渐变背景 + 动态模糊
- **卡片**: `backdrop-filter: blur()` 实现玻璃态
- **动画**: 流畅的 CSS 过渡
- **色彩**: CSS 变量系统

### 主题系统

- **亮色主题**: 蓝紫色调
- **暗色主题**: 深色背景 + 柔和高亮
- **持久化**: localStorage 保存偏好
- **管理**: Pinia store 统一管理

## 🔧 开发指南

详细开发文档请查看 [README.dev.md](./README.dev.md)，包含：

- 快速开始
- 命令参考
- 构建与打包
- 项目结构详解
- 开发工作流
- 调试技巧
- 常见问题

## 📝 脚本命令

### Bun Scripts

```bash
# 开发
bun run dev              # Vite 开发服务器
bun run dev:web          # Web 模式
bun run tauri:dev        # Tauri 开发模式

# 构建
bun run build            # Vite 构建
bun run build:web        # Web 构建
bun run build:desktop    # Desktop 完整构建

# 检查
bun run lint             # ESLint
bun run type-check       # TypeScript
bun run tauri:check      # Cargo check
bun run check:all        # 全面检查

# 工具
bun run clean            # 清理构建
bun run clean:all        # 深度清理
```

### Just Commands

```bash
just dev                 # Tauri 开发
just build               # Desktop 构建
just check-all           # 全面检查
just clean               # 清理
just fresh               # 清理 + 安装 + 开发
just ci                  # 检查 + 构建
```

## 📦 打包产物

### Linux
- `.deb` - Debian/Ubuntu 包
- `.AppImage` - 通用 AppImage

### macOS
- `.dmg` - 安装镜像
- `.app` - 应用程序包

### Windows
- `.msi` - 安装程序
- `.exe` - 可执行文件

产物位置: `src-tauri/target/release/bundle/`

## 🚀 部署

### Web 部署

```bash
bun run build:web
# 将 dist/ 目录部署到静态服务器
```

### Desktop 分发

```bash
bun run build:desktop
# 在 src-tauri/target/release/bundle/ 中找到安装包
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 开发流程

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 📄 许可证

MIT License - 详见 [LICENSE](../../LICENSE)

## 🙏 致谢

- [CCR Core](https://github.com/bahayonghang/ccr) - 核心库
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [Vite](https://vitejs.dev/) - 构建工具

## 📚 相关文档

- [开发文档](./README.dev.md) - 完整开发指南
- [更新日志](./CHANGELOG.md) - 版本历史
- [CCR 主仓库](https://github.com/bahayonghang/ccr) - 核心项目

---

Made with ❤️ using Vue.js 3, Tauri 2.0, TypeScript, and Liquid Glass Design

**版本**: 2.5.0 | **更新**: 2025-11-08
