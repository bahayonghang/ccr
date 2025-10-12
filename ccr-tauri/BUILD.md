# CCR Desktop - 开发和构建指南

## 🏗️ 快速构建命令

```bash
# 1. 在根项目安装 Rust 依赖 (只需一次)
cd /path/to/ccr
cargo check

# 2. 进入 Tauri 子项目
cd ccr-tauri

# 3. 安装前端依赖 (只需一次)
cd src-ui
npm install
cd ..

# 4. 开发模式运行
cargo tauri dev

# 5. 构建生产版本
cargo tauri build
```

## 🔧 详细步骤

### 步骤 1: 环境准备

确保已安装：
- Rust 1.70+ (`rustup update`)
- Node.js 18+ (`node --version`)
- npm 9+ (`npm --version`)

### 步骤 2: 系统依赖

#### macOS

```bash
xcode-select --install
```

#### Linux (Ubuntu/Debian)

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

#### Windows

WebView2 (Windows 10/11 自带，无需额外安装)

### 步骤 3: 前端依赖安装

```bash
cd ccr-tauri/src-ui
npm install
```

这将安装：
- Vue 3
- TypeScript
- Vite
- Tauri API 包
- 其他前端依赖

### 步骤 4: 开发模式

#### 方式 1: 使用 Cargo (推荐)

```bash
cd ccr-tauri
cargo tauri dev
```

#### 方式 2: 使用 npm

```bash
cd ccr-tauri/src-ui
npm run tauri:dev
```

**开发模式特性：**
- ✅ Rust 代码热重载 (需重新编译)
- ✅ Vue 3 热模块替换 (HMR)
- ✅ TypeScript 类型检查
- ✅ Vue DevTools
- ✅ Tauri DevTools (F12)

### 步骤 5: 生产构建

```bash
cd ccr-tauri
cargo tauri build
```

**构建产物位置：**

```
ccr-tauri/target/release/bundle/
├── macos/
│   ├── CCR Desktop.app           # macOS 应用包
│   └── dmg/
│       └── CCR Desktop_1.1.2_x64.dmg    # macOS 安装镜像
├── appimage/
│   └── ccr-desktop_1.1.2_amd64.AppImage # Linux AppImage
├── deb/
│   └── ccr-desktop_1.1.2_amd64.deb      # Debian 包
└── msi/
    └── CCR Desktop_1.1.2_x64_en-US.msi  # Windows 安装包
```

## 🎯 常用命令

### 开发命令

```bash
# 前端开发服务器 (仅 UI，无后端)
cd src-ui && npm run dev

# 前端类型检查
cd src-ui && npm run build

# Rust 代码检查
cd ccr-tauri && cargo check

# Rust 代码格式化
cd ccr-tauri && cargo fmt

# Rust 代码 Lint
cd ccr-tauri && cargo clippy
```

### 清理命令

```bash
# 清理 Rust 构建缓存
cd ccr-tauri && cargo clean

# 清理前端构建产物
cd src-ui && rm -rf dist node_modules

# 完全清理 (重新开始)
cd ccr-tauri
cargo clean
cd src-ui
rm -rf dist node_modules package-lock.json
npm install
```

## 📦 发布清单

发布新版本前的检查项：

- [ ] 更新版本号 (4 个文件):
  - [ ] `ccr-tauri/Cargo.toml`
  - [ ] `ccr-tauri/src-ui/package.json`
  - [ ] `ccr-tauri/tauri.conf.json`
  - [ ] 根项目 `Cargo.toml`
- [ ] 运行所有测试: `cargo test --all`
- [ ] 构建成功: `cargo tauri build`
- [ ] 在 3 个平台测试 (macOS, Linux, Windows)
- [ ] 更新 CHANGELOG.md
- [ ] 创建 Git tag: `git tag -a ccr-tauri-v1.1.2`
- [ ] 发布到 GitHub Releases

## 🔍 调试技巧

### Rust 后端调试

1. **启用详细日志：**

```bash
export RUST_LOG=ccr_tauri=trace,ccr=debug,tauri=debug
cargo tauri dev
```

2. **使用 LLDB/GDB 调试器：**

```bash
# macOS/Linux
rust-lldb target/debug/ccr-tauri

# Windows
rust-gdb target/debug/ccr-tauri.exe
```

### 前端调试

1. **浏览器 DevTools (F12)**
   - Console: 查看日志和错误
   - Network: 查看 Tauri Command 调用
   - Vue DevTools: 查看组件状态

2. **Vite 开发服务器日志：**

```bash
cd src-ui
npm run dev
# 查看 http://localhost:5173
```

### 性能分析

```bash
# 构建带调试信息的 Release 版本
cargo tauri build --debug

# 使用系统性能分析工具
# macOS: Instruments
# Linux: perf
# Windows: Windows Performance Analyzer
```

## 🚨 故障排查

### 问题: `cargo tauri dev` 启动失败

**可能原因 1: 端口占用**

```bash
# 检查 5173 端口
lsof -i :5173
# 或修改 vite.config.ts 中的端口号
```

**可能原因 2: 依赖未安装**

```bash
cd src-ui
rm -rf node_modules package-lock.json
npm install
```

### 问题: 构建时找不到 CCR 核心库

确保在 Workspace 根目录：

```bash
cd ccr  # 根项目目录
cargo check --all
cd ccr-tauri
cargo tauri build
```

### 问题: Tauri Command 调用失败

1. 检查 Command 注册 (`src/main.rs`)
2. 检查参数类型匹配
3. 查看控制台错误信息:

```javascript
// 前端 src-ui/src/api/index.ts
try {
  await invoke('my_command', { param })
} catch (error) {
  console.error('Command failed:', error)
}
```

## 📚 参考资源

- [Tauri 官方文档](https://tauri.app/v2/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Vite 文档](https://vitejs.dev/)
- [CCR 主项目文档](../CLAUDE.md)

---

**祝你构建顺利！有问题随时查看这份文档～ (￣▽￣)／**
