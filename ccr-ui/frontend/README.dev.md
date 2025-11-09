# CCR Desktop - Development Guide

CCR桌面应用开发指南

## 📋 目录

- [快速开始](#快速开始)
- [开发命令](#开发命令)
- [构建与打包](#构建与打包)
- [项目结构](#项目结构)
- [技术栈](#技术栈)

## 🚀 快速开始

### 环境要求

- **Node.js**: >= 18.0.0
- **Rust**: >= 1.70
- **系统要求**: Linux / macOS / Windows
- **可选**: [just](https://github.com/casey/just) 命令运行器

### 安装依赖

```bash
# 进入前端目录
cd ccr-ui/frontend

# 安装 npm 依赖
npm install

# 或使用 justfile
just install
```

### 启动开发模式

```bash
# 方法 1: 使用 npm
npm run tauri:dev

# 方法 2: 使用 just（推荐）
just dev

# 方法 3: 仅前端开发（无 Tauri）
npm run dev:web
just dev-web
```

首次启动会编译 Rust 后端（约 5-10 分钟），后续启动会快很多。

## 🛠️ 开发命令

### NPM Scripts

| 命令 | 说明 |
|------|------|
| `npm run dev` | 启动 Vite 开发服务器 |
| `npm run build` | 构建前端生产版本 |
| `npm run preview` | 预览构建结果 |
| `npm run lint` | ESLint 检查和修复 |
| `npm run type-check` | TypeScript 类型检查 |
| `npm run tauri:dev` | Tauri 开发模式（推荐） |
| `npm run tauri:build` | 构建桌面应用 |
| `npm run tauri:check` | 快速检查 Rust 代码 |
| `npm run tauri:clippy` | Rust 代码质量检查 |
| `npm run tauri:fmt` | 格式化 Rust 代码 |
| `npm run tauri:test` | 运行 Rust 测试 |
| `npm run check:all` | 全面检查（TS + Lint + Cargo） |
| `npm run clean` | 清理构建产物 |
| `npm run clean:all` | 深度清理（含 node_modules） |

### Just Commands

[Just](https://github.com/casey/just) 提供更简洁的命令：

```bash
# 查看所有命令
just

# 开发
just dev              # Tauri 开发模式
just dev-web          # Web 开发模式
just dev-frontend     # 仅前端

# 构建
just build            # 完整桌面应用
just build-web        # Web 版本
just build-debug      # 调试版本

# 检查
just check            # Cargo check
just clippy           # Cargo clippy
just lint             # ESLint
just type-check       # TypeScript
just check-all        # 全面检查

# 清理
just clean            # 清理构建产物
just clean-all        # 深度清理
just reset            # 重置并重装依赖

# 组合命令
just fresh            # 清理 + 安装 + 开发
just ci               # 检查 + 构建
just proto            # 格式化 + 检查 + 开发
```

### 🖥️ Tauri 专用命令（推荐使用 ccr-ui 根目录的 justfile）

在 `ccr-ui/` 根目录下，我们添加了完整的 Tauri 命令支持：

```bash
# 开发
cd ../  # 回到 ccr-ui 根目录
just tauri-dev        # 启动 Tauri 开发模式
just tauri-check      # 检查 Tauri 环境

# 构建
just tauri-build      # 构建生产版本
just tauri-build-debug # 构建调试版本（更快）

# 代码质量
just tauri-check-all  # 完整检查（TS + Rust）
just tauri-check-rust # 只检查 Rust 代码
just tauri-clippy     # Rust Clippy linter
just tauri-fmt        # 格式化 Rust 代码
just tauri-test       # 运行 Tauri 测试

# 清理
just tauri-clean      # 清理 Tauri 构建产物
```

**为什么推荐使用根目录的 justfile？**
- ✅ 统一的命令入口（Web + Desktop）
- ✅ 跨平台支持（Linux/macOS/Windows）
- ✅ 更简洁的命令名称
- ✅ 与 backend 命令一致

## 📦 构建与打包

### 开发构建（快速）

```bash
npm run tauri:build:debug
# 或
just build-debug
```

- 更快的编译速度
- 包含调试符号
- 文件体积较大
- 适合测试

### 生产构建（优化）

```bash
npm run build:desktop
# 或
just build
```

- 完整优化
- 体积更小
- 性能更好
- 发布使用

### 构建产物位置

```
src-tauri/target/release/
├── ccr-desktop              # 可执行文件（Linux/macOS）
├── ccr-desktop.exe          # 可执行文件（Windows）
└── bundle/
    ├── deb/                 # Linux .deb 包
    │   └── ccr-desktop_2.5.0_amd64.deb
    ├── appimage/            # Linux AppImage
    │   └── ccr-desktop_2.5.0_amd64.AppImage
    ├── dmg/                 # macOS .dmg 安装包
    │   └── CCR Desktop_2.5.0_x64.dmg
    └── msi/                 # Windows .msi 安装包
        └── CCR Desktop_2.5.0_x64_en-US.msi
```

### 平台特定打包

```bash
# 仅构建特定目标
tauri build --target deb        # Linux Debian 包
tauri build --target appimage   # Linux AppImage
tauri build --target dmg        # macOS DMG
tauri build --target msi        # Windows MSI
```

## 📁 项目结构

```
ccr-ui/frontend/
├── src/                        # Vue.js 源码
│   ├── api/                    # API 客户端
│   │   ├── client.ts           # HTTP API
│   │   ├── tauri.ts            # Tauri API
│   │   └── index.ts            # 统一入口
│   ├── components/             # Vue 组件
│   ├── views/                  # 页面视图
│   ├── router/                 # Vue Router
│   ├── store/                  # Pinia Store
│   ├── types/                  # TypeScript 类型
│   ├── styles/                 # 全局样式
│   ├── App.vue                 # 根组件
│   └── main.ts                 # 入口文件
├── src-tauri/                  # Tauri Rust 后端
│   ├── src/
│   │   └── main.rs             # Rust 主程序
│   ├── icons/                  # 应用图标
│   ├── Cargo.toml              # Rust 依赖
│   ├── tauri.conf.json         # Tauri 配置
│   └── build.rs                # 构建脚本
├── public/                     # 静态资源
├── dist/                       # 构建输出（生成）
├── package.json                # npm 配置
├── justfile                    # Just 命令
├── vite.config.ts              # Vite 配置
├── tsconfig.json               # TypeScript 配置
└── README.dev.md               # 本文档
```

## 🔧 技术栈

### 前端

- **框架**: Vue.js 3.5 (Composition API)
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
- **核心库**: CCR (path dependency)
- **异步**: Tokio 1.48
- **序列化**: Serde + Serde JSON
- **日志**: Tracing

### 开发工具

- **包管理**: npm / pnpm / yarn
- **代码检查**: ESLint + Clippy
- **格式化**: Prettier + rustfmt
- **类型检查**: vue-tsc
- **命令运行**: Just

## 🎯 开发工作流

### 1. 功能开发

```bash
# 1. 创建功能分支
git checkout -b feature/my-feature

# 2. 启动开发模式
just dev

# 3. 修改代码（热重载生效）
# - 前端修改: 自动刷新
# - Rust 修改: 自动重新编译

# 4. 提交代码
just commit "feat: add my feature"
```

### 2. 代码质量

```bash
# 运行所有检查
just check-all

# 或分别运行
just type-check    # TypeScript 类型
just lint          # ESLint 代码风格
just check         # Rust 编译检查
just clippy        # Rust 代码质量
```

### 3. 发布流程

```bash
# 1. 更新版本号
# - package.json: "version": "2.6.0"
# - src-tauri/Cargo.toml: version = "2.6.0"
# - src-tauri/tauri.conf.json: "version": "2.6.0"

# 2. 运行完整检查
just ci

# 3. 构建发布版本
just build

# 4. 测试安装包
# 找到 src-tauri/target/release/bundle/ 中的安装包

# 5. 提交并打标签
git add -A
git commit -m "chore: release v2.6.0"
git tag v2.6.0
git push && git push --tags
```

## 🐛 调试技巧

### 前端调试

1. **Chrome DevTools**
   - Tauri 窗口右键 → 检查元素
   - 或按 `F12` 打开

2. **Vue DevTools**
   ```bash
   # 安装 Vue DevTools 扩展
   # Chrome/Edge: https://devtools.vuejs.org
   ```

3. **Console 日志**
   ```typescript
   console.log('Debug info:', data)
   console.error('Error:', error)
   ```

### Rust 后端调试

1. **日志输出**
   ```rust
   tracing::info!("Info message");
   tracing::error!("Error: {:?}", error);
   ```

2. **环境变量**
   ```bash
   # 设置日志级别
   export RUST_LOG=debug
   just dev
   ```

3. **调试构建**
   ```bash
   # 使用调试版本（保留符号）
   just build-debug
   ```

## 📖 相关文档

- [Tauri 官方文档](https://tauri.app/v1/guides/)
- [Vue.js 文档](https://vuejs.org/)
- [Vite 文档](https://vitejs.dev/)
- [Just 文档](https://just.systems/)
- [CCR 项目主页](https://github.com/bahayonghang/ccr)

## ❓ 常见问题

### Q: Tauri 编译失败？

**A**: 检查 Rust 工具链：
```bash
rustc --version  # 应该 >= 1.70
cargo --version
```

### Q: 前端热重载不工作？

**A**: 重启开发服务器：
```bash
just clean
just dev
```

### Q: 图标不显示？

**A**: 检查图标文件：
```bash
ls -lh src-tauri/icons/
# 应该有 32x32.png, 128x128.png, icon.png 等
```

### Q: Tauri 命令调用失败？

**A**: 检查命令是否在 `tauri::generate_handler!` 中注册：
```rust
// src-tauri/src/main.rs
.invoke_handler(tauri::generate_handler![
    list_profiles,
    switch_profile,
    // ...
])
```

---

**作者**: CCR Team
**最后更新**: 2025-11-08
**版本**: 2.5.0
